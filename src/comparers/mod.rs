mod normal;
mod strict;

pub use self::normal::NormalComparer;
pub use self::strict::StrictComparer;

use crate::byte_reader::{ByteReader, ByteReaderLike};
use crate::compare::{CompareTask, Comparer, Comparison};
use crate::one_mut::OneMut;
use std::fs::File;
use std::io::stdin;
use std::os::unix::io::AsRawFd;

pub trait ByteComparer {
    fn compare(&self, std: &mut impl ByteReaderLike, user: &mut impl ByteReaderLike) -> Comparison;

    fn exec(&self, task: &CompareTask) -> Comparison {
        let (mut user_file, user_in, mut user_in_lock);

        let user_fd: &mut dyn AsRawFd = match task.user_path.as_ref() {
            Some(path) => match File::open(path) {
                Ok(file) => {
                    user_file = file;
                    &mut user_file
                }
                Err(err) => {
                    let code = err.raw_os_error().unwrap();
                    crate::error::exit(code, err)
                }
            },
            None => {
                user_in = stdin();
                user_in_lock = user_in.lock();
                &mut user_in_lock
            }
        };

        let mut std_file;

        let std_fd: &mut File = match File::open(&task.std_path) {
            Ok(file) => {
                std_file = file;
                &mut std_file
            }
            Err(err) => {
                let code = err.raw_os_error().unwrap();
                crate::error::exit(code, err)
            }
        };

        const CAP: usize = 64 * 1024;
        static USER_BUF: OneMut<[u8; CAP]> = OneMut::new([0u8; CAP]);
        static STD_BUF: OneMut<[u8; CAP]> = OneMut::new([0u8; CAP]);

        let mut user_buf_lock = USER_BUF.get_mut().unwrap();
        let mut std_buf_lock = STD_BUF.get_mut().unwrap();

        let mut user_reader = ByteReader::new(user_fd, &mut *user_buf_lock);
        let mut std_reader = ByteReader::new(std_fd, &mut *std_buf_lock);

        let ans = Self::compare(self, &mut std_reader, &mut user_reader);

        if task.user_read_all {
            user_reader.wait_all()
        }

        ans
    }
}

impl<T: ByteComparer> Comparer for T {
    fn exec(&self, task: &CompareTask) -> Comparison {
        <Self as ByteComparer>::exec(&self, task)
    }
}
