mod normal;

#[deny(unsafe_code)]
mod strict;

#[deny(unsafe_code)]
mod float;

pub use float::float_compare;
pub use normal::normal_compare;
pub use strict::strict_compare;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    AC,
    WA,
    PE,
}
