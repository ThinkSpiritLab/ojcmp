use super::ByteComparer;
use crate::byte_reader::ByteReaderLike;
use crate::compare::Comparison;
use std::cmp;

pub struct StrictComparer {
    _priv: (),
}

impl StrictComparer {
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl ByteComparer for StrictComparer {
    fn compare(&self, std: &mut impl ByteReaderLike, user: &mut impl ByteReaderLike) -> Comparison {
        self::compare(std, user)
    }
}

/// Compare `std` and `user`. The process will be terminated on error.
fn compare(std: &mut impl ByteReaderLike, user: &mut impl ByteReaderLike) -> Comparison {
    let mut std_buf = std.next_buf();
    let mut user_buf = user.next_buf();

    loop {
        match (std_buf, user_buf) {
            (None, None) => return Comparison::AC,
            (Some(_), None) | (None, Some(_)) => return Comparison::WA,

            (Some(lhs), Some(rhs)) => match lhs.len().cmp(&rhs.len()) {
                cmp::Ordering::Equal => {
                    if lhs != rhs {
                        return Comparison::WA;
                    }
                    std_buf = std.next_buf();
                    user_buf = user.next_buf();
                }
                cmp::Ordering::Less => {
                    let (rhs, remaining) = rhs.split_at(lhs.len());
                    if lhs != rhs {
                        return Comparison::WA;
                    }
                    std_buf = std.next_buf();
                    user_buf = Some(remaining);
                }
                cmp::Ordering::Greater => {
                    let (lhs, remaining) = lhs.split_at(rhs.len());
                    if lhs != rhs {
                        return Comparison::WA;
                    }
                    std_buf = Some(remaining);
                    user_buf = user.next_buf();
                }
            },
        }
    }
}

#[test]
fn test_strict_comparer() {
    macro_rules! judge {
        ($ret:expr, $std:expr,$user:expr) => {{
            let mut std = std::io::Cursor::new(&$std[..]);
            let mut user = std::io::Cursor::new(&$user[..]);

            let ret = compare(&mut std, &mut user);
            assert_eq!(ret, $ret);
        }};
    }

    use crate::compare::Comparison::*;

    judge!(AC, b"", b"");
    judge!(AC, b"1", b"1");
    judge!(AC, b"12", b"12");

    judge!(WA, b"", b"a");
    judge!(WA, b"a", b"");
    judge!(WA, b"ab", b"ba");
    judge!(WA, b"cc", b"ccc");
    judge!(WA, b"ccc", b"cc");
}
