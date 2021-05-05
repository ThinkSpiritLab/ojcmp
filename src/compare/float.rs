use super::{catch_io, CompareError, Comparison};

use crate::byte_read::ByteRead;

use std::panic::AssertUnwindSafe;

pub fn try_float_compare(
    std_reader: &mut impl ByteRead,
    user_reader: &mut impl ByteRead,
    eps: f64,
) -> Result<Comparison, CompareError> {
    catch_io(AssertUnwindSafe(move || {
        float_compare(std_reader, user_reader, eps)
    }))
    .map_err(CompareError::Io)
}

fn float_compare(
    std_reader: &mut impl ByteRead,
    user_reader: &mut impl ByteRead,
    eps: f64,
) -> Comparison {
    loop {
        let std_f64 = match poll_f64(std_reader) {
            Ok(o) => o,
            Err(()) => return Comparison::WA,
        };

        let user_f64 = match poll_f64(user_reader) {
            Ok(o) => o,
            Err(()) => return Comparison::WA,
        };

        match (std_f64, user_f64) {
            (Some(_), None) | (None, Some(_)) => return Comparison::WA,

            (None, None) => return Comparison::AC,

            (Some(a), Some(b)) => {
                let diff = (b - a).abs(); // check nan or +inf !!!
                if let None | Some(std::cmp::Ordering::Greater) = diff.partial_cmp(&eps) {
                    return Comparison::WA;
                }
            }
        }
    }
}

fn poll_f64(reader: &mut impl ByteRead) -> Result<Option<f64>, ()> {
    let mut buf: [u8; 512] = [0; 512];
    let mut cur: usize = 0;

    let mut byte = reader.next_byte();
    loop {
        if byte.is_eof() {
            return Ok(None);
        } else {
            let b = byte.as_u8();
            if !b.is_ascii_whitespace() {
                buf[cur] = b;
                cur += 1;
                byte = reader.next_byte();
                break;
            }
        }
        byte = reader.next_byte();
    }

    while cur < buf.len() {
        if byte.is_eof() {
            break;
        } else {
            let b = byte.as_u8();
            if b.is_ascii_whitespace() {
                break;
            }
            buf[cur] = b;
            cur += 1;
        }
        byte = reader.next_byte();
    }

    if cur >= buf.len() {
        return Err(());
    }
    if cur == 0 {
        return Ok(None);
    }
    match lexical_core::parse::<f64>(&buf[..cur]) {
        Ok(x) => Ok(Some(x)),
        Err(_) => Err(()),
    }
}

#[test]
fn test_spj_float_comparer() {
    const DEFAULT_EPS: f64 = 1e-10;

    macro_rules! judge {
        ($ret:expr, $std:expr,$user:expr) => {{
            let mut std: &[u8] = $std.as_ref();
            let mut user: &[u8] = $user.as_ref();

            let ret = float_compare(&mut std, &mut user, DEFAULT_EPS);
            assert_eq!(ret, $ret);
        }};
    }

    use Comparison::*;

    judge!(AC, b"", b"");
    judge!(AC, b"1", b"1");
    judge!(AC, b"12", b"12");
    judge!(AC, b"12 34", b"12 34");

    judge!(WA, b"", b"a");
    judge!(WA, b"a", b"");
    judge!(WA, b"ab", b"ba");
    judge!(WA, b"cc", b"ccc");
    judge!(WA, b"ccc", b"cc");

    judge!(WA, b"1.0", b"1.0000000001");
    judge!(WA, b"1.0", b"0.9999999999");

    judge!(AC, b"1.0", b"1.00000000009");
    judge!(AC, b"1.0", b"0.99999999991");

    judge!(WA, b"1.0", b"nan");
    judge!(WA, b"nan", b"1.0");

    judge!(WA, b"0.0", b"-inf");
    judge!(WA, b"0.0", b"+inf");
}
