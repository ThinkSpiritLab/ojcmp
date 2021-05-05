#![deny(missing_debug_implementations)]

mod byte_read;
mod compare;

pub use byte_read::{ByteRead, ByteReader};
pub use compare::Comparison;
pub use compare::{try_float_compare, try_normal_compare, try_strict_compare};

#[cfg(unix)]
pub use byte_read::unix::UnixFdReader;
