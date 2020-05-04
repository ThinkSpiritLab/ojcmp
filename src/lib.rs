#![deny(unsafe_code)]

#[cfg(not(target_os = "linux"))]
compile_error!("ojcmp only supports linux");

pub mod compare;
pub mod comparers;
pub mod error;

#[allow(unsafe_code)]
pub mod byte_reader;

#[allow(unsafe_code)]
pub mod one_mut;
