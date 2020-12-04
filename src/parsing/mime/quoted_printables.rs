// Second rule of the encoding
fn litteral_repr_possible(c: u8) -> bool {
    (c >= 33 && c <= 60) || (c >= 62 && c <= 126)
}

#[allow(clippy::if_same_then_else)]
pub fn encode_qp(mut data: Vec<u8>) -> Vec<u8> {
    // Fixme: Make it usable by binary formats
    let mut line_lenght = 0;
    let mut idx = 0;

    while let Some(byte) = data.get(idx).copied() {
        if line_lenght >= 72 {
            // 72 because in the worst case we add 3 chars + 1 equal sign -> 76
            // Fifth rule
            data.insert(idx, b'\n');
            data.insert(idx, b'\r');
            data.insert(idx, b'=');
            idx += 3;
            line_lenght = 0;
        }

        if litteral_repr_possible(byte) {
            // Second rule
            idx += 1;
            line_lenght += 1;
        } else if (byte == 9 || byte == 32)
            && data.get(idx + 1).map(|c| c != &b'\r').unwrap_or(false)
        {
            // Third rule
            idx += 1;
            line_lenght += 1;
        } else if byte == b'\r' && data.get(idx + 1) == Some(&b'\n') {
            // Fourth rule
            idx += 2;
            line_lenght = 0;
        } else {
            // First rule

            fn to_hex(n: u8) -> u8 {
                match n {
                    0..=9 => b'0' + n,
                    10..=15 => b'A' + n - 10,
                    _ => unreachable!(),
                }
            }

            data.remove(idx);
            data.insert(idx, to_hex(byte % 16));
            data.insert(idx, to_hex((byte - byte % 16) / 16));
            data.insert(idx, b'=');

            idx += 3;
            line_lenght += 3;
        };
    }

    data
}

pub fn decode_qp(mut data: Vec<u8>) -> Vec<u8> {
    let mut idx = 0;

    while let Some(byte) = data.get(idx).copied() {
        if litteral_repr_possible(byte) || byte == b' ' || byte == b'\t' {
            idx += 1;
        } else if byte == b'=' {
            if data.get(idx + 1) == Some(&b'\r') && data.get(idx + 2) == Some(&b'\n') {
                data.remove(idx);
                data.remove(idx);
                data.remove(idx);
            } else if data.len() > idx + 2 {
                let first = data.remove(idx + 1);
                let second = data.remove(idx + 1);

                fn from_hex(n: u8) -> Option<u8> {
                    match n {
                        b'0'..=b'9' => Some(n - b'0'),
                        b'A'..=b'F' => Some(10 + n - b'A'),
                        b'a'..=b'f' => Some(10 + n - b'a'),
                        _ => None,
                    }
                }

                if let (Some(first), Some(second)) = (from_hex(first), from_hex(second)) {
                    data[idx] = first * 16 + second;
                    idx += 1;
                } else {
                    data[idx] = 189;
                    data.insert(idx, 191);
                    data.insert(idx, 239);
                    idx += 3;
                }
            } else {
                idx += 1;
            }
        } else if byte == b'\r' && data.get(idx + 1) == Some(&b'\n') {
            idx += 2;
        } else {
            data[idx] = 189;
            data.insert(idx, 191);
            data.insert(idx, 239);
            idx += 3;
        }
    }

    data
}

pub fn decode_header_qp(mut data: Vec<u8>) -> Vec<u8> {
    let mut idx = 0;

    while let Some(byte) = data.get(idx).copied() {
        if byte == b'_' {
            data[idx] = 0x20;
            idx += 1;
        } else if byte == b'=' {
            if data.get(idx + 1) == Some(&b'\r') && data.get(idx + 2) == Some(&b'\n') {
                data.remove(idx);
                data.remove(idx);
                data.remove(idx);
            } else if data.len() > idx + 2 {
                let first = data.remove(idx + 1);
                let second = data.remove(idx + 1);

                fn from_hex(n: u8) -> Option<u8> {
                    match n {
                        b'0'..=b'9' => Some(n - b'0'),
                        b'A'..=b'F' => Some(10 + n - b'A'),
                        b'a'..=b'f' => Some(10 + n - b'a'),
                        _ => None,
                    }
                }

                if let (Some(first), Some(second)) = (from_hex(first), from_hex(second)) {
                    data[idx] = first * 16 + second;
                    idx += 1;
                } else {
                    data[idx] = 189;
                    data.insert(idx, 191);
                    data.insert(idx, 239);
                    idx += 3;
                }
            } else {
                idx += 1;
            }
        } else if byte >= 0x20 && byte <= 0x7E {
            idx += 1;
        } else {
            data[idx] = 189;
            data.insert(idx, 191);
            data.insert(idx, 239);
            idx += 3;
        }
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode() {
        assert_eq!(
            b"This was a triumph. Il =C3=A9tait une fois...\r\nAnd voil=C3=A0 !=20\r\nSimon"
                .to_vec(),
            encode_qp(
                "This was a triumph. Il était une fois...\r\nAnd voilà ! \r\nSimon"
                    .to_string()
                    .into_bytes()
            )
        );
        assert_eq!(b"Now\'s the time for all folk to come to the aid of their country. Wtf thi=\r\ns sentence is not long enough to test line-lenght limit.".to_vec(), encode_qp("Now's the time for all folk to come to the aid of their country. Wtf this sentence is not long enough to test line-lenght limit.".to_string().into_bytes()));
    }

    #[test]
    fn decode() {
        assert_eq!(
            b"This was a triumph. Il \xC3\xA9tait une fois...\r\nAnd voil\xC3\xA0 ! \r\nSimon"
                .to_vec(),
            decode_qp(
                "This was a triumph. Il =C3=A9tait une fois...\r\nAnd voil=C3=A0 !=20\r\nSimon"
                    .to_string()
                    .into_bytes()
            )
        );
        assert_eq!(b"Now's the time for all folk to come to the aid of their country. Wtf this sentence is not long enough to test line-lenght limit.".to_vec(), decode_qp("Now\'s the time for all folk to come to the aid of their country. Wtf thi=\r\ns sentence is not long enough to test line-lenght limit.".to_string().into_bytes()));
    }
}
