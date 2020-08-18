use crate::byte_read::{ByteRead, IoByte};
use crate::Comparison;
use std::cmp::{Ord, Ordering};

/// Compare `std` and `user`. The process will be terminated on error.
#[inline(never)]
pub fn normal_compare(
    std_reader: &mut impl ByteRead,
    user_reader: &mut impl ByteRead,
) -> Comparison {
    let mut std_byte = std_reader.next_byte();
    let mut user_byte = user_reader.next_byte();

    let mut ans = Comparison::AC;

    loop {
        if std_byte.is_eof() {
            return handle_eof(user_reader, user_byte, ans);
        }

        if user_byte.is_eof() {
            return handle_eof(std_reader, std_byte, ans);
        }

        let (a, b) = (std_byte.as_u8(), user_byte.as_u8());
        if a == b {
            let ret = poll_diff(std_reader, user_reader);
            std_byte = ret.0;
            user_byte = ret.1;
            continue;
        }

        if a == b'\n' {
            if !b.is_ascii_whitespace() {
                return Comparison::WA;
            }
            if poll_endline(user_reader) {
                std_byte = std_reader.next_byte();
                user_byte = user_reader.next_byte();
                continue;
            } else {
                return Comparison::WA;
            }
        }
        if b == b'\n' {
            if !a.is_ascii_whitespace() {
                return Comparison::WA;
            }
            if poll_endline(std_reader) {
                std_byte = std_reader.next_byte();
                user_byte = user_reader.next_byte();
                continue;
            } else {
                return Comparison::WA;
            }
        }

        let flaga = a.is_ascii_whitespace();
        let flagb = b.is_ascii_whitespace();

        // a != b
        // both of them are non-space
        if !flaga & !flagb {
            return Comparison::WA;
        }

        // a != b
        // both of them are not non-space
        if flaga {
            std_byte = poll_nonspace(std_reader);
        }
        if flagb {
            user_byte = poll_nonspace(user_reader);
        }

        if std_byte.is_eof() || user_byte.is_eof() {
            continue;
        }

        let (a, b) = (std_byte.as_u8(), user_byte.as_u8());
        let flaga = a == b'\n';
        let flagb = b == b'\n';

        if flaga & flagb {
            std_byte = std_reader.next_byte();
            user_byte = user_reader.next_byte();
            continue;
        }
        if flaga | flagb {
            return Comparison::WA;
        }
        if a == b {
            ans = Comparison::PE;
            std_byte = std_reader.next_byte();
            user_byte = user_reader.next_byte();
            continue;
        } else {
            return Comparison::WA;
        }
    }
}

#[inline(never)]
fn handle_eof(rhs: &mut impl ByteRead, rhs_byte: IoByte, ans: Comparison) -> Comparison {
    if rhs_byte.is_eof() {
        return ans;
    }
    if !rhs_byte.as_u8().is_ascii_whitespace() {
        return Comparison::WA;
    }
    if poll_eof(rhs) {
        ans
    } else {
        Comparison::WA
    }
}

#[inline]
fn poll_diff(lhs: &mut impl ByteRead, rhs: &mut impl ByteRead) -> (IoByte, IoByte) {
    let mut lhs_byte;
    let mut rhs_byte;
    let mut eq_cnt: usize = 0;
    let mut cmp_cnt: usize = 0;

    loop {
        lhs_byte = lhs.next_byte();
        rhs_byte = rhs.next_byte();
        cmp_cnt += 1;

        if cmp_cnt >= 1024 {
            break;
        }

        if lhs_byte == rhs_byte {
            eq_cnt += 1;
            if lhs_byte.is_eof() {
                return (lhs_byte, rhs_byte);
            }
        } else {
            return (lhs_byte, rhs_byte);
        }
    }

    loop {
        if cmp_cnt >= 1024 && eq_cnt > cmp_cnt * 255 / 256 {
            let len = diff_block(lhs, rhs);
            if len == 0 {
                eq_cnt = 0;
                cmp_cnt = 0;
            } else {
                eq_cnt += len;
                cmp_cnt += len;
            }
        }

        lhs_byte = lhs.next_byte();
        rhs_byte = rhs.next_byte();
        cmp_cnt += 1;

        if lhs_byte == rhs_byte {
            eq_cnt += 1;
            if lhs_byte.is_eof() {
                return (lhs_byte, rhs_byte);
            }
        } else {
            return (lhs_byte, rhs_byte);
        }
    }
}

#[inline]
fn diff_block(lhs: &mut impl ByteRead, rhs: &mut impl ByteRead) -> usize {
    let mut total: usize = 0;
    loop {
        let lhs_buf: &[u8] = match lhs.fill_buf() {
            Ok(b) => b,
            Err(e) => panic!(e),
        };

        let rhs_buf: &[u8] = match rhs.fill_buf() {
            Ok(b) => b,
            Err(e) => panic!(e),
        };

        let (lhs_buf, rhs_buf, len) = match lhs_buf.len().cmp(&rhs_buf.len()) {
            Ordering::Equal => (lhs_buf, rhs_buf, lhs_buf.len()),
            Ordering::Less => (lhs_buf, &rhs_buf[..lhs_buf.len()], lhs_buf.len()),
            Ordering::Greater => (&lhs_buf[..rhs_buf.len()], rhs_buf, rhs_buf.len()),
        };

        if len == 0 {
            break total;
        }

        if lhs_buf == rhs_buf {
            lhs.consume(len);
            rhs.consume(len);
            total += len;
            continue;
        } else {
            break 0;
        }
    }
}

/// poll until eof.
/// ensure that all chars remaining in `chars` are ascii whitespaces
#[inline]
fn poll_eof(reader: &mut impl ByteRead) -> bool {
    loop {
        let b = reader.next_byte();
        if b.is_eof() {
            return true;
        }
        if !b.as_u8().is_ascii_whitespace() {
            return false;
        }
    }
}

/// poll until b'\n'.
/// ensure that all chars remaining in `chars` line are ascii whitespaces
#[inline(always)]
fn poll_endline(reader: &mut impl ByteRead) -> bool {
    let mut b = reader.next_byte();
    loop {
        if b.is_eof() || b.as_u8() == b'\n' {
            return true;
        }
        if !b.as_u8().is_ascii_whitespace() {
            return false;
        }
        b = reader.next_byte();
    }
}

/// poll until b'\n' or non-space or EOF
#[inline(always)]
fn poll_nonspace(reader: &mut impl ByteRead) -> IoByte {
    loop {
        let b: IoByte = reader.next_byte();
        if b.is_eof() || b.as_u8() == b'\n' || !b.as_u8().is_ascii_whitespace() {
            return b;
        }
    }
}

#[test]
fn test_normal_comparer() {
    macro_rules! judge {
        ($ret:expr, $std:expr,$user:expr) => {{
            let mut std: &[u8] = $std.as_ref();
            let mut user: &[u8] = $user.as_ref();

            let ret = normal_compare(&mut std, &mut user);
            assert_eq!(ret, $ret);
        }};
    }

    use Comparison::*;

    judge!(WA, b"1", b"2");
    judge!(WA, b"1\r\n", b"2\n");
    judge!(PE, b"1\r3\n", b"1\t3\n");
    judge!(PE, b"1 3\n", b"1\t3\n");
    judge!(PE, b"1 3\n", b"1         3\n");
    judge!(PE, b"1 3\r\n", b"1         3\r\n");
    judge!(PE, b"1 3\r\n", b"1         3\n");
    judge!(PE, b"1 3\n", b"1         3\r\n");
    judge!(PE, b"1\r3\t4\n", b"1\r3\r4\r\n");
    judge!(AC, b"1 2\n3 4", b"1 2\r\n3 4\n");
    judge!(AC, b"1 2 \n3 4", b"1 2 \r\n3 4 \n");
    judge!(AC, b"\n", b"");
    judge!(AC, b"", b"\n");
    judge!(AC, b" \n", b" ");
    judge!(AC, b"1\n", b"1");
    judge!(AC, b"1 \n", b"1");
    judge!(AC, b"1 \n", b"1\n");
    judge!(AC, b"1\t\n", b"1\r\n");
    judge!(AC, b"1\r\n", b"1\r");
    judge!(AC, b"1 2  \n3 4", b"1 2    \t\n3 4");
    judge!(AC, b"1 2 \r\n3 4", b"1 2                  \r\n3 4");
    judge!(AC, b"1\r\n\r\n\r\n", b"1  ");
    judge!(AC, b"1\r\n2\r\n", b"1 \n2 \n");
    judge!(AC, b"1\r\n2\r\n", b"1 \n2\t\n");
    judge!(AC, b"\t\n1", b"\r\n1");
    judge!(WA, b"asd", b"qwe");
    judge!(PE, b" asd", b"  asd");
    judge!(WA, b" asd", b"\nasd");
    judge!(PE, b" asd  \n", b"\tasd  \n");
    judge!(WA, b" asd  2\n", b"\tasd  1\n");
    judge!(AC, b"1\r", b"1\t");
    judge!(WA, b"1\na", b"1\n");
    judge!(WA, b"1\n", b"1\na");
    judge!(WA, b"1a", b"1");
    judge!(WA, b"1", b"1a");
    judge!(WA, b"1a \nb", b"1  \nb");
    judge!(WA, b"1\naa", b"1\n");
    judge!(WA, b"1\n", b"1\naa");
    judge!(WA, b"1 a", b"1 ");
    judge!(WA, b"1 aa", b"1 ");
    judge!(WA, b"1 ", b"1 a");
    judge!(WA, b"1 ", b"1 aa");
    judge!(AC, b"1\n\n3\n", b"1\r\n  \r\n3\t\n");
    judge!(WA, b"1\n3\n", b"1\r\n  \r\n3\t\n");
}
