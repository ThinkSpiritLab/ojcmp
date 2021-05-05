mod normal;

#[deny(unsafe_code)]
mod strict;

#[deny(unsafe_code)]
mod float;

use std::panic::{catch_unwind, resume_unwind, UnwindSafe};
use std::{fmt, io, panic};

pub use self::float::try_float_compare;
pub use self::normal::try_normal_compare;
pub use self::strict::try_strict_compare;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    AC = 0,
    WA = 1,
    PE = 2,
}

#[derive(Debug)]
pub enum CompareError {
    Io(io::Error),
}

impl fmt::Display for CompareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompareError::Io(e) => {
                write!(f, "CompareError::Io: {}", e)
            }
        }
    }
}

impl std::error::Error for CompareError {}

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
