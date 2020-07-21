mod normal;
pub use normal::normal_compare;

mod strict;
pub use strict::strict_compare;

mod float;
pub use float::float_compare;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    AC,
    WA,
    PE,
}
