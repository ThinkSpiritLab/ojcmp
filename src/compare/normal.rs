use crate::byte_read::{ByteRead, IoByte};
use crate::Comparison;

/// Compare `std` and `user`. The process will be terminated on error.
#[inline]
pub fn normal_compare(std: &mut impl ByteRead, user: &mut impl ByteRead) -> Comparison {
    let mut stdchar = std.next_byte();
    let mut userchar = user.next_byte();

    let mut ans = Comparison::AC;

    loop {
        if stdchar.is_eof() {
            if userchar.is_eof() {
                return ans;
            }
            if !userchar.as_u8().is_ascii_whitespace() {
                return Comparison::WA;
            }
            if poll_eof(user) {
                return ans;
            } else {
                return Comparison::WA;
            }
        }

        if userchar.is_eof() {
            if stdchar.is_eof() {
                return ans;
            }
            if !stdchar.as_u8().is_ascii_whitespace() {
                return Comparison::WA;
            }
            if poll_eof(std) {
                return ans;
            } else {
                return Comparison::WA;
            }
        }

        let (a, b) = (stdchar.as_u8(), userchar.as_u8());
        if a == b {
            stdchar = std.next_byte();
            userchar = user.next_byte();
            continue;
        }

        if a == b'\n' {
            if !b.is_ascii_whitespace() {
                return Comparison::WA;
            }
            if poll_endline(user) {
                stdchar = std.next_byte();
                userchar = user.next_byte();
                continue;
            } else {
                return Comparison::WA;
            }
        }
        if b == b'\n' {
            if !a.is_ascii_whitespace() {
                return Comparison::WA;
            }
            if poll_endline(std) {
                stdchar = std.next_byte();
                userchar = user.next_byte();
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
            stdchar = poll_nonspace(std);
        }
        if flagb {
            userchar = poll_nonspace(user);
        }

        if stdchar.is_eof() || userchar.is_eof() {
            continue;
        }

        let (a, b) = (stdchar.as_u8(), userchar.as_u8());
        let flaga = a == b'\n';
        let flagb = b == b'\n';

        if flaga & flagb {
            stdchar = std.next_byte();
            userchar = user.next_byte();
            continue;
        }
        if flaga | flagb {
            return Comparison::WA;
        }
        if a == b {
            ans = Comparison::PE;
            stdchar = std.next_byte();
            userchar = user.next_byte();
            continue;
        } else {
            return Comparison::WA;
        }
    }
}

/// poll until eof.
/// ensure that all chars remaining in `chars` are ascii whitespaces
#[inline]
fn poll_eof(bytes: &mut impl ByteRead) -> bool {
    let mut b = bytes.next_byte();
    loop {
        if b.is_eof() {
            return true;
        }
        if !b.as_u8().is_ascii_whitespace() {
            return false;
        }
        b = bytes.next_byte();
    }
}

/// poll until b'\n'.
/// ensure that all chars remaining in `chars` line are ascii whitespaces
#[inline]
fn poll_endline(bytes: &mut impl ByteRead) -> bool {
    let mut b = bytes.next_byte();
    loop {
        if b.is_eof() || b.as_u8() == b'\n' {
            return true;
        }
        if !b.as_u8().is_ascii_whitespace() {
            return false;
        }
        b = bytes.next_byte();
    }
}

/// poll until b'\n' or non-space or EOF
#[inline]
fn poll_nonspace(bytes: &mut impl ByteRead) -> IoByte {
    let mut b: IoByte = bytes.next_byte();
    loop {
        if b.is_eof() || b.as_u8() == b'\n' || !b.as_u8().is_ascii_whitespace() {
            return b;
        }
        b = bytes.next_byte();
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
