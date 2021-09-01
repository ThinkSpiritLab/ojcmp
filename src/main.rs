#[cfg(not(target_os = "linux"))]
compile_error!("ojcmp does not support this platform now");

use ojcmp::{ByteReader, Comparison};

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::prelude::FromRawFd;
use std::path::PathBuf;
use std::process;
use structopt::clap::ArgGroup;
use structopt::StructOpt;

use anyhow::{Context, Result};

#[derive(Debug, StructOpt)]
#[structopt(author)]
enum Opts {
    /// Normal compare
    Normal {
        #[structopt(flatten)]
        common_opts: CommonOpts,
    },
    /// Strict compare
    Strict {
        #[structopt(flatten)]
        common_opts: CommonOpts,
    },
    /// Float compare
    Float {
        #[structopt(flatten)]
        common_opts: CommonOpts,

        #[structopt(name = "eps", short = "e", long)]
        /// Eps for float comparing
        eps: f64,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(
    group(ArgGroup::with_name("std_file").args(&["std", "std-fd"]).required(true)),
    group(ArgGroup::with_name("user_file").args(&["user", "user-fd"]).required(true)),
)]
struct CommonOpts {
    /// Std file path
    #[structopt(short = "s", long, value_name = "path")]
    std: Option<PathBuf>,

    /// Std file descriptor
    #[structopt(long, value_name = "fd")]
    std_fd: Option<i32>,

    /// User file path
    #[structopt(short = "u", long, value_name = "path")]
    user: Option<PathBuf>,

    /// User file descriptor
    #[structopt(long, value_name = "fd")]
    user_fd: Option<i32>,

    /// Reads all bytes of user file even if it's already WA
    #[structopt(short = "a", long)]
    read_all: bool,

    /// Buffer size (in bytes) for both std and user file
    #[structopt(short = "b", long, default_value = "65536", value_name = "bytes")]
    buffer_size: usize,

    /// No output printed to stdout or stderr
    #[structopt(short = "q", long)]
    quiet: bool,
}

fn open(common_opts: &CommonOpts) -> anyhow::Result<(File, File)> {
    let std_file = match (&common_opts.std, common_opts.std_fd) {
        (Some(p), _) => {
            File::open(p).with_context(|| format!("failed to open std file: {:?}", p))?
        }
        (None, Some(fd)) => unsafe { File::from_raw_fd(fd) },
        (None, None) => anyhow::bail!("std file must be specified"),
    };

    let user_file = match (&common_opts.user, common_opts.user_fd) {
        (Some(p), _) => {
            File::open(p).with_context(|| format!("failed to open user file: {:?}", p))?
        }
        (None, Some(fd)) => unsafe { File::from_raw_fd(fd) },
        (None, None) => anyhow::bail!("user file must be specified"),
    };

    anyhow::ensure!(
        common_opts.buffer_size >= 1024,
        "buffer size is too small: buffer_size = {}",
        common_opts.buffer_size
    );

    Ok((std_file, user_file))
}

fn consume_all(reader: &mut impl BufRead) -> Result<()> {
    loop {
        let buf = reader.fill_buf()?;
        let amt = buf.len();
        if amt == 0 {
            break Ok(());
        }
        reader.consume(amt);
    }
}

#[repr(align(16))]
struct Align16<T>(T);

const BUF_SIZE: usize = 512 * 1024;
static mut STD_BUF: Align16<[u8; BUF_SIZE]> = Align16([0u8; BUF_SIZE]);
static mut USER_BUF: Align16<[u8; BUF_SIZE]> = Align16([0u8; BUF_SIZE]);

fn handle_normal(common_opts: &CommonOpts) -> Result<Comparison> {
    let (std_file, user_file) = open(common_opts)?;

    let (mut std_reader, mut user_reader) = {
        #[cfg(unix)]
        let std_file = ojcmp::UnixFdReader::from_file(std_file);

        #[cfg(unix)]
        let user_file = ojcmp::UnixFdReader::from_file(user_file);

        if common_opts.buffer_size <= BUF_SIZE {
            unsafe {
                (
                    ByteReader::from_raw(&mut STD_BUF.0[..common_opts.buffer_size], std_file),
                    ByteReader::from_raw(&mut USER_BUF.0[..common_opts.buffer_size], user_file),
                )
            }
        } else {
            (
                ByteReader::with_capacity(common_opts.buffer_size, std_file),
                ByteReader::with_capacity(common_opts.buffer_size, user_file),
            )
        }
    };

    let ans = ojcmp::try_normal_compare(&mut std_reader, &mut user_reader)?;

    if common_opts.read_all {
        consume_all(&mut user_reader)?;
    }

    if common_opts.buffer_size <= BUF_SIZE {
        unsafe {
            let _ = std_reader.into_raw();
            let _ = user_reader.into_raw();
        }
    }

    Ok(ans)
}

fn handle_strict(common_opts: &CommonOpts) -> Result<Comparison> {
    let (std_file, user_file) = open(common_opts)?;
    let mut std_reader = BufReader::with_capacity(common_opts.buffer_size, std_file);
    let mut user_reader = BufReader::with_capacity(common_opts.buffer_size, user_file);

    let ans = ojcmp::try_strict_compare(&mut std_reader, &mut user_reader)?;

    if common_opts.read_all {
        consume_all(&mut user_reader)?;
    }

    Ok(ans)
}

fn handle_float(common_opts: &CommonOpts, eps: f64) -> Result<Comparison> {
    let (std_file, user_file) = open(common_opts)?;
    let mut std_reader = ByteReader::with_capacity(common_opts.buffer_size, std_file);
    let mut user_reader = ByteReader::with_capacity(common_opts.buffer_size, user_file);

    anyhow::ensure!(
        (eps == 0.0 || eps.is_normal()) && !eps.is_nan(),
        "eps is invalid: eps = {}",
        eps
    );

    anyhow::ensure!(eps >= 0.0, "eps must be non-negative: eps = {}", eps);

    let ans = ojcmp::try_float_compare(&mut std_reader, &mut user_reader, eps)?;

    if common_opts.read_all {
        consume_all(&mut user_reader)?;
    }

    Ok(ans)
}

fn main() {
    let opts: Opts = Opts::from_args();

    let (common_opts, ret) = match opts {
        Opts::Normal { ref common_opts } => {
            (common_opts, handle_normal(common_opts)) //
        }
        Opts::Strict { ref common_opts } => {
            (common_opts, handle_strict(common_opts)) //
        }
        Opts::Float {
            ref common_opts,
            eps,
        } => {
            (common_opts, handle_float(common_opts, eps)) //
        }
    };

    let exit_code = match ret {
        Ok(ans) => {
            if !common_opts.quiet {
                let output = match ans {
                    Comparison::AC => "AC",
                    Comparison::WA => "WA",
                    Comparison::PE => "PE",
                };

                println!("{}", output);
            }
            ans as i32
        }
        Err(err) => {
            if !common_opts.quiet {
                eprintln!("{}", err);
            }
            101
        }
    };

    process::exit(exit_code)
}
