use super::ByteComparer;
use crate::byte_reader::ByteReaderLike;
use crate::compare::Comparison;

pub struct SpjFloatComparer {
    eps: f64,
}

impl SpjFloatComparer {
    pub fn new(eps: f64) -> Self {
        Self { eps }
    }
}

impl ByteComparer for SpjFloatComparer {
    fn compare(&self, std: &mut impl ByteReaderLike, user: &mut impl ByteReaderLike) -> Comparison {
        self::compare(std, user, self.eps)
    }
}

/// Compare `std` and `user`. The process will be terminated on error.
fn compare(std: &mut impl ByteReaderLike, user: &mut impl ByteReaderLike, eps: f64) -> Comparison {
    loop {
        let std_f64 = match poll_f64(std) {
            Ok(o) => o,
            Err(()) => return Comparison::WA,
        };

        let user_f64 = match poll_f64(user) {
            Ok(o) => o,
            Err(()) => return Comparison::WA,
        };

        match (std_f64, user_f64) {
            (Some(_), None) | (None, Some(_)) => return Comparison::WA,

            (None, None) => return Comparison::AC,

            (Some(a), Some(b)) => {
                let diff = (b - a).abs();
                if diff >= eps {
                    return Comparison::WA;
                }
            }
        }
    }
}

fn poll_f64(bytes: &mut impl ByteReaderLike) -> Result<Option<f64>, ()> {
    let mut buf: [u8; 512] = [0; 512];
    let mut cur: usize = 0;

    let mut b = bytes.next_byte();
    loop {
        if b.is_eof() {
            return Ok(None);
        }
        if !b.as_u8().is_ascii_whitespace() {
            buf[cur] = b.as_u8();
            cur += 1;
            b = bytes.next_byte();
            break;
        }
        b = bytes.next_byte();
    }

    while cur < buf.len() {
        if b.is_eof() {
            break;
        }
        let c = b.as_u8();
        if c.is_ascii_whitespace() {
            break;
        }
        buf[cur] = c;
        cur += 1;
        b = bytes.next_byte();
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
            let mut std = std::io::Cursor::new(&$std[..]);
            let mut user = std::io::Cursor::new(&$user[..]);

            let ret = compare(&mut std, &mut user, DEFAULT_EPS);
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

    judge!(WA, b"1.0", b"1.0000000001");
    judge!(WA, b"1.0", b"0.9999999999");

    judge!(AC, b"1.0", b"1.00000000009");
    judge!(AC, b"1.0", b"0.99999999991");
}
