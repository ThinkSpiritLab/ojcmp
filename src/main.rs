#[cfg(not(target_os = "linux"))]
compile_error!("ojcmp only supports linux");

pub mod chars;
pub mod compare;

#[cfg(test)]
pub mod test;

pub mod errno {
    pub unsafe fn get_errno() -> i32 {
        *libc::__errno_location() as _
    }

    use std::ffi::CStr;
    pub unsafe fn on_error() -> ! {
        let errno = get_errno();
        let ptr = libc::strerror(errno); // non-null
        let msg = CStr::from_ptr(ptr);
        eprintln!("{}", msg.to_str().unwrap());
        std::process::exit(errno)
    }
}

use crate::chars::Chars;
use crate::compare::compare;
use crate::errno::on_error;

use std::ffi::CString;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::io::RawFd;
use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(name = "std", short = "s", long, value_name = "path")]
    /// Std file path.
    std_path: PathBuf,

    #[structopt(name = "user", short = "u", long, value_name = "path")]
    /// User file path. Read from stdin if it's not given.
    user_path: Option<PathBuf>,

    #[structopt(name = "all", short = "a", long)]
    /// Read all bytes of user file even if it's already WA.
    read_all: bool,
}

fn open_ro(path: PathBuf) -> RawFd {
    let path = CString::new(path.into_os_string().into_vec()).unwrap();
    let path = path.as_ptr();
    unsafe {
        let ret = libc::open(path, libc::O_RDONLY);
        if ret < 0 {
            on_error()
        }
        ret as _
    }
}

fn main() {
    let opt: Opt = Opt::from_args();

    let user_fd = match opt.user_path {
        Some(path) => open_ro(path),
        None => libc::STDIN_FILENO,
    };
    let std_fd = open_ro(opt.std_path);

    const CAP: usize = 64 * 1024;
    static mut STD_BUF: [u8; CAP] = [0; CAP];
    static mut USER_BUF: [u8; CAP] = [0; CAP];

    let mut std = unsafe { Chars::with_buf(&mut STD_BUF[..], std_fd) };
    let mut user = unsafe { Chars::with_buf(&mut USER_BUF[..], user_fd) };

    let cmp = compare(&mut std, &mut user);

    if opt.read_all {
        user.drop_all();
    }

    println!("{:?}", cmp);
}
