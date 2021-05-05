use super::{CompareError, Comparison};

use std::cmp::Ordering;
use std::io::{self, BufRead};

pub fn try_strict_compare(
    std_reader: &mut impl BufRead,
    user_reader: &mut impl BufRead,
) -> Result<Comparison, CompareError> {
    strict_compare(std_reader, user_reader).map_err(CompareError::Io)
}

fn fill_buf(reader: &mut impl BufRead) -> io::Result<Option<&[u8]>> {
    let buf = reader.fill_buf()?;
    if buf.is_empty() {
        Ok(None)
    } else {
        Ok(Some(buf))
    }
}

fn strict_compare(
    std_reader: &mut impl BufRead,
    user_reader: &mut impl BufRead,
) -> io::Result<Comparison> {
    loop {
        let std_buf = fill_buf(std_reader)?;
        let user_buf = fill_buf(user_reader)?;
        let std_len;
        let user_len;

        match (std_buf, user_buf) {
            (None, None) => return Ok(Comparison::AC),
            (Some(_), None) | (None, Some(_)) => return Ok(Comparison::WA),
            (Some(lhs), Some(rhs)) => match lhs.len().cmp(&rhs.len()) {
                Ordering::Equal => {
                    if lhs != rhs {
                        return Ok(Comparison::WA);
                    }
                    std_len = lhs.len();
                    user_len = rhs.len();
                }
                Ordering::Less => {
                    let rhs = &rhs[..lhs.len()];
                    if lhs != rhs {
                        return Ok(Comparison::WA);
                    }
                    std_len = lhs.len();
                    user_len = rhs.len();
                }
                Ordering::Greater => {
                    let lhs = &lhs[..rhs.len()];
                    if lhs != rhs {
                        return Ok(Comparison::WA);
                    }
                    std_len = lhs.len();
                    user_len = rhs.len();
                }
            },
        }
        std_reader.consume(std_len);
        user_reader.consume(user_len);
    }
}

#[test]
fn test_strict_comparer() {
    macro_rules! judge {
        ($ret:expr, $std:expr,$user:expr) => {{
            let mut std: &[u8] = $std.as_ref();
            let mut user: &[u8] = $user.as_ref();

            let ret = strict_compare(&mut std, &mut user).unwrap();
            assert_eq!(ret, $ret);
        }};
    }

    use Comparison::*;

    judge!(AC, b"", b"");
    judge!(AC, b"1", b"1");
    judge!(AC, b"12", b"12");

    judge!(WA, b"", b"a");
    judge!(WA, b"a", b"");
    judge!(WA, b"ab", b"ba");
    judge!(WA, b"cc", b"ccc");
    judge!(WA, b"ccc", b"cc");
}
