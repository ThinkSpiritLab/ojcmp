use super::Comparison;
use crate::byte_read::ByteRead;

pub fn normal_compare(
    std_reader: &mut impl ByteRead,
    user_reader: &mut impl ByteRead,
) -> Comparison {
    let mut std_byte = std_reader.next_byte();
    let mut user_byte = user_reader.next_byte();

    let mut ans = Comparison::AC;

    loop {
        match (std_byte, user_byte) {
            (None, None) => {
                return ans;
            }
            (None, Some(user_char)) => {
                if !user_char.is_ascii_whitespace() {
                    return Comparison::WA;
                }
                if poll_eof(user_reader) {
                    return ans;
                } else {
                    return Comparison::WA;
                }
            }
            (Some(std_char), None) => {
                if !std_char.is_ascii_whitespace() {
                    return Comparison::WA;
                }
                if poll_eof(std_reader) {
                    return ans;
                } else {
                    return Comparison::WA;
                }
            }
            (Some(std_char), Some(user_char)) => {
                if std_char == user_char {
                    std_byte = std_reader.next_byte();
                    user_byte = user_reader.next_byte();
                    continue;
                }

                if std_char == b'\n' {
                    if !user_char.is_ascii_whitespace() {
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

                if user_char == b'\n' {
                    if !std_char.is_ascii_whitespace() {
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

                let std_is_whitespace = std_char.is_ascii_whitespace();
                let user_is_whitespace = user_char.is_ascii_whitespace();

                if !std_is_whitespace & !user_is_whitespace {
                    return Comparison::WA;
                }

                if std_is_whitespace {
                    std_byte = poll_nonspace(std_reader);
                }
                if user_is_whitespace {
                    user_byte = poll_nonspace(user_reader);
                }

                match (std_byte, user_byte) {
                    (None, _) | (_, None) => continue,
                    (Some(a), Some(b)) => match (a, b) {
                        (b'\n', b'\n') => {
                            std_byte = std_reader.next_byte();
                            user_byte = user_reader.next_byte();
                            continue;
                        }
                        (b'\n', _) | (_, b'\n') => {
                            return Comparison::WA;
                        }
                        _ if a == b => {
                            ans = Comparison::PE;
                            std_byte = std_reader.next_byte();
                            user_byte = user_reader.next_byte();
                            continue;
                        }
                        _ => {
                            return Comparison::WA;
                        }
                    },
                }
            }
        }
    }
}

fn poll_eof(reader: &mut impl ByteRead) -> bool {
    let mut byte = reader.next_byte();
    loop {
        match byte {
            None => return true,
            Some(b) => {
                if !b.is_ascii_whitespace() {
                    return false;
                }
            }
        };
        byte = reader.next_byte();
    }
}

fn poll_endline(reader: &mut impl ByteRead) -> bool {
    let mut byte = reader.next_byte();
    loop {
        match byte {
            None | Some(b'\n') => return true,
            Some(b) => {
                if !b.is_ascii_whitespace() {
                    return false;
                }
            }
        }
        byte = reader.next_byte();
    }
}

fn poll_nonspace(reader: &mut impl ByteRead) -> Option<u8> {
    let mut byte = reader.next_byte();
    loop {
        match byte {
            None | Some(b'\n') => return byte,
            Some(b) => {
                if !b.is_ascii_whitespace() {
                    return byte;
                }
            }
        }
        byte = reader.next_byte();
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
