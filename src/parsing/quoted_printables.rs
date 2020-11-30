// Second rule of the encoding
fn litteral_repr_possible(c: u8) -> bool {
    (c >= 33 && c <= 60) || (c >= 62 && c <= 126)
}

#[allow(clippy::if_same_then_else)]
pub fn encode_qp(input: String) -> String {
    let mut input = input.into_bytes();
    let mut line_lenght = 0;
    let mut idx = 0;

    while let Some(byte) = input.get(idx).copied() {
        if line_lenght >= 72 { // 72 because in the worst case we add 3 chars + 1 equal sign -> 76
            // Fifth rule
            input.insert(idx, b'\r');
            input.insert(idx, b'\n');
            input.insert(idx, b'=');
            idx += 3;
            line_lenght = 0;
        }

        if litteral_repr_possible(byte) {
            // Second rule
            idx += 1;
            line_lenght += 1;
        } else if (byte == 9 || byte == 32)
            && input
                .get(idx + 1)
                .map(|c| c != &b'\r')
                .unwrap_or(false)
        {
            // Third rule
            idx += 1;
            line_lenght += 1;
        } else if byte == b'\r' && input.get(idx + 1) == Some(&b'\n') {
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

            input.remove(idx);
            input.insert(idx, to_hex(byte % 16));
            input.insert(idx, to_hex((byte - byte % 16) / 16));
            input.insert(idx, b'=');

            idx += 3;
            line_lenght += 3;
        };
    }

    unsafe { String::from_utf8_unchecked(input) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode() {
        assert_eq!("This was a triumph. Il =C3=A9tait une fois...\r\nAnd voil=C3=A0 !=20\r\nSimon", encode_qp("This was a triumph. Il était une fois...\r\nAnd voilà ! \r\nSimon".to_string()));
        assert_eq!("Now\'s the time for all folk to come to the aid of their country. Wtf thi=\n\rs sentence is not long enough to test line-lenght limit.", encode_qp("Now's the time for all folk to come to the aid of their country. Wtf this sentence is not long enough to test line-lenght limit.".to_string()));
    }
}
