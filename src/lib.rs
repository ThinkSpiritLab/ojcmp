#![deny(unsafe_code)]
#![deny(missing_debug_implementations)]

#![allow(unsafe_code)]
mod byte_read;

mod compare;

pub use byte_read::{ByteRead, ByteReader};

pub use compare::Comparison;
pub use compare::{float_compare, normal_compare, strict_compare};
