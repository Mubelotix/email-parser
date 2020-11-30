// Second rule of the encoding
fn litteral_repr_possible(c: u8) -> bool {
    (c >= 33 && c <= 60) || (c >= 62 && c <= 126)
}

#[allow(clippy::if_same_then_else)]
pub fn encode_qp(input: String) -> String {
    let mut output = Vec::new();
    let mut bytes = input.bytes().peekable();
    let mut line_lenght = 0;

    while let Some(byte) = bytes.next() {
        if line_lenght >= 72 { // 72 because in the worst case we add 3 chars + 1 equal sign -> 76
            // Fifth rule
            output.push(b'=');
            output.push(b'\r');
            output.push(b'\n');
            line_lenght = 0;
        }

        line_lenght = if litteral_repr_possible(byte) {
            // Second rule
            output.push(byte);
            line_lenght + 1
        } else if (byte == 9 || byte == 32)
            && bytes
                .peek()
                .map(|c| c != &b'\r')
                .unwrap_or(false)
        {
            // Third rule
            output.push(byte);
            line_lenght + 1
        } else if byte == b'\r' && bytes.peek() == Some(&b'\n') {
            // Fourth rule
            bytes.next();
            output.push(b'\r');
            output.push(b'\n');
            0
        } else {
            // First rule

            fn to_hex(n: u8) -> u8 {
                match n {
                    0..=9 => b'0' + n,
                    10..=15 => b'A' + n - 10,
                    _ => unreachable!(),
                }
            }

            output.push(b'=');
            output.push(to_hex((byte - byte % 16) / 16));
            output.push(to_hex(byte % 16));
            line_lenght + 3
        }
    }
    unsafe { String::from_utf8_unchecked(output) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode() {
        println!("{:?}", encode_qp("This was a triumph. Il était une fois...\r\nAnd voilà ! \r\nSimon".to_string()));
        println!("{:?}", encode_qp("Now's the time for all folk to come to the aid of their country. Wtf this sentence is not long enough to test line-lenght limit.".to_string()));
    }
}
