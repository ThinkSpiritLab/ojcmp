use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::mem;
use std::panic::{self, catch_unwind, resume_unwind, AssertUnwindSafe, UnwindSafe};
use std::path::PathBuf;
use structopt::StructOpt;

use ojcmp::{ByteReader, Comparison};

#[derive(Debug, StructOpt)]
#[structopt(author)]
enum Opts {
    /// Normal compare
    Normal {
        #[structopt(flatten)]
        reader_opts: ReaderOpts,
    },
    /// Strict compare
    Strict {
        #[structopt(flatten)]
        reader_opts: ReaderOpts,
    },
    /// Float compare
    Float {
        #[structopt(flatten)]
        reader_opts: ReaderOpts,

        #[structopt(name = "eps", short = "e", long)]
        /// Eps for float comparing
        eps: f64,
    },
}

#[derive(Debug, StructOpt)]
struct ReaderOpts {
    #[structopt(name = "std", short = "s", long, value_name = "path")]
    /// Std file path
    std: PathBuf,
    #[structopt(name = "user", short = "u", long, value_name = "path")]
    /// User file path
    user: PathBuf,
    #[structopt(name = "read-all", short = "a", long)]
    /// Reads all bytes of user file even if it's already WA
    read_all: bool,

    #[structopt(
        name = "buffer-size",
        short = "b",
        long,
        default_value = "4096",
        value_name = "bytes"
    )]
    /// Buffer size (in bytes) for both std and user file
    buffer_size: usize,
}

fn catch_io<R>(f: impl FnOnce() -> R + UnwindSafe) -> io::Result<R> {
    let hook = panic::take_hook();
    let ret = match catch_unwind(f) {
        Ok(ans) => Ok(ans),
        Err(payload) => match payload.downcast::<io::Error>() {
            Ok(e) => Err(*e),
            Err(payload) => resume_unwind(payload),
        },
    };
    panic::set_hook(hook);
    ret
}

fn open(reader_opts: &ReaderOpts) -> anyhow::Result<(File, File)> {
    let std_file = File::open(&reader_opts.std)
        .with_context(|| format!("Failed to open std file: {:?}", reader_opts.std))?;

    let user_file = File::open(&reader_opts.user)
        .with_context(|| format!("Failed to open user file: {:?}", reader_opts.user))?;

    anyhow::ensure!(
        reader_opts.buffer_size >= 1024,
        "buffer size is too small: buffer_size = {}",
        reader_opts.buffer_size
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

#[repr(align(8))]
struct Align8<T>(T);

const BUF_SIZE: usize = 512 * 1024;
static mut STD_BUF: Align8<[u8; BUF_SIZE]> = Align8([0u8; BUF_SIZE]);
static mut USER_BUF: Align8<[u8; BUF_SIZE]> = Align8([0u8; BUF_SIZE]);

fn main() -> Result<()> {
    let opts: Opts = Opts::from_args();

    let ans: Comparison;

    match opts {
        Opts::Normal { reader_opts } => {
            let (std_file, user_file) = open(&reader_opts)?;

            let (mut std_reader, mut user_reader) = {
                if reader_opts.buffer_size <= BUF_SIZE {
                    unsafe {
                        (
                            ByteReader::from_raw(
                                &mut STD_BUF.0[..reader_opts.buffer_size],
                                std_file,
                            ),
                            ByteReader::from_raw(
                                &mut USER_BUF.0[..reader_opts.buffer_size],
                                user_file,
                            ),
                        )
                    }
                } else {
                    (
                        ByteReader::with_capacity(reader_opts.buffer_size, std_file),
                        ByteReader::with_capacity(reader_opts.buffer_size, user_file),
                    )
                }
            };

            ans = catch_io(AssertUnwindSafe(|| {
                ojcmp::normal_compare(&mut std_reader, &mut user_reader)
            }))?;

            if reader_opts.read_all {
                consume_all(&mut user_reader)?;
            }

            if reader_opts.buffer_size <= BUF_SIZE {
                mem::forget(std_reader);
                mem::forget(user_reader);
            }
        }
        Opts::Strict { reader_opts } => {
            let (std_file, user_file) = open(&reader_opts)?;
            let mut std_reader = BufReader::with_capacity(reader_opts.buffer_size, std_file);
            let mut user_reader = BufReader::with_capacity(reader_opts.buffer_size, user_file);

            ans = ojcmp::strict_compare(&mut std_reader, &mut user_reader)?;

            if reader_opts.read_all {
                consume_all(&mut user_reader)?;
            }
        }
        Opts::Float { reader_opts, eps } => {
            let (std_file, user_file) = open(&reader_opts)?;
            let mut std_reader = ByteReader::with_capacity(reader_opts.buffer_size, std_file);
            let mut user_reader = ByteReader::with_capacity(reader_opts.buffer_size, user_file);

            anyhow::ensure!(
                (eps == 0.0 || eps.is_normal()) && !eps.is_nan(),
                "eps is invalid: eps = {}",
                eps
            );

            anyhow::ensure!(eps >= 0.0, "eps must be non-negative: eps = {}", eps);

            ans = catch_io(AssertUnwindSafe(|| {
                ojcmp::float_compare(&mut std_reader, &mut user_reader, eps)
            }))?;

            if reader_opts.read_all {
                consume_all(&mut user_reader)?;
            }
        }
    }

    let output = match ans {
        Comparison::AC => "AC",
        Comparison::WA => "WA",
        Comparison::PE => "PE",
    };

    println!("{}", output);

    Ok(())
}
