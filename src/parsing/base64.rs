const BASE64_MAP: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];

pub fn encode_base64(data: Vec<u8>) -> Vec<u8> {
    let mut encoded_data = Vec::new();
    let mut bytes = data.iter();
    let mut line_lenght = 0;

    while let Some(byte1) = bytes.next() {
        if line_lenght >= 72 { // 76 - 4 = 72
            encoded_data.push(b'\r');
            encoded_data.push(b'\n');
            line_lenght = 0;
        }
        match (bytes.next(), bytes.next()) {
            (Some(byte2), Some(byte3)) => {
                let output_byte1 = (0b11111100 & byte1) >> 2;
                let output_byte2 = ((0b00000011 & byte1) << 4) + ((0b11110000 & byte2) >> 4);
                let output_byte3 = ((0b00001111 & byte2) << 2) + ((0b11000000 & byte3) >> 6);
                let output_byte4 = 0b00111111 & byte3;
                encoded_data.push(unsafe { *BASE64_MAP.get_unchecked(output_byte1 as usize) });
                encoded_data.push(unsafe { *BASE64_MAP.get_unchecked(output_byte2 as usize) });
                encoded_data.push(unsafe { *BASE64_MAP.get_unchecked(output_byte3 as usize) });
                encoded_data.push(unsafe { *BASE64_MAP.get_unchecked(output_byte4 as usize) });
            }
            (Some(byte2), None) => {
                let output_byte1 = (0b11111100 & byte1) >> 2;
                let output_byte2 = ((0b00000011 & byte1) << 4) + ((0b11110000 & byte2) >> 4);
                let output_byte3 = (0b00001111 & byte2) << 2;
                encoded_data.push(unsafe { *BASE64_MAP.get_unchecked(output_byte1 as usize) });
                encoded_data.push(unsafe { *BASE64_MAP.get_unchecked(output_byte2 as usize) });
                encoded_data.push(unsafe { *BASE64_MAP.get_unchecked(output_byte3 as usize) });
                encoded_data.push(b'=');
            }
            (None, None) => {
                let output_byte1 = (0b11111100 & byte1) >> 2;
                let output_byte2 = (0b00000011 & byte1) << 4;
                encoded_data.push(unsafe { *BASE64_MAP.get_unchecked(output_byte1 as usize) });
                encoded_data.push(unsafe { *BASE64_MAP.get_unchecked(output_byte2 as usize) });
                encoded_data.push(b'=');
                encoded_data.push(b'=');
            }
            _ => unreachable!(),
        }
        line_lenght += 4;
    }

    encoded_data
}

fn get_value_encoded(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(26 + (c - b'a')),
        b'0'..=b'9' => Some(26 * 2 + (c - b'0')),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

pub fn decode_base64(data: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    let mut bytes = data.iter();
    let mut decoded_data = Vec::new();

    'main: loop {
        let b1 = 'inner1: loop {
            match bytes.next() {
                Some(b) => {
                    if let Some(b) = get_value_encoded(*b) {
                        break 'inner1 b;
                    }
                }
                None => break 'main,
            }
        };

        let b2 = 'inner2: loop {
            match bytes.next() {
                Some(b) => {
                    if let Some(b) = get_value_encoded(*b) {
                        break 'inner2 b;
                    }
                }
                None => return Err("Missing at least 3 bytes"),
            }
        };

        let b3 = 'inner3: loop {
            match bytes.next() {
                Some(b) if *b == b'=' => break 'inner3 None,
                Some(b) => {
                    if let Some(b) = get_value_encoded(*b) {
                        break 'inner3 Some(b);
                    }
                }
                None => return Err("Missing at least 2 bytes"),
            }
        };

        let b4 = 'inner4: loop {
            match bytes.next() {
                Some(b) if *b == b'=' => break 'inner4 None,
                Some(_) if b3.is_none() => return Err("Data after end of data"),
                Some(b) => {
                    if let Some(b) = get_value_encoded(*b) {
                        break 'inner4 Some(b);
                    }
                }
                None => return Err("Missing at least 1 byte"),
            }
        };

        decoded_data.push((b1 << 2) + ((b2 & 0b00110000) >> 4));
        if let Some(b3) = b3 {
            decoded_data.push(((b2 & 0b00001111) << 4) + ((b3 & 0b00111100) >> 2));

            if let Some(b4) = b4 {
                decoded_data.push(((b3 & 0b00000011) << 6) + b4);
            };
        };
    }

    Ok(decoded_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode() {
        assert_eq!(
            "VGhhdCdzIGEgdGVzdCE=",
            String::from_utf8(encode_base64(b"That's a test!".to_vec())).unwrap()
        );
        assert_eq!(
            "UnVzdCBpcyB0aGUgYmVzdCBsYW5ndWFnZQ==",
            String::from_utf8(encode_base64(b"Rust is the best language".to_vec())).unwrap()
        );
        assert_eq!(
            "SSBhbSBmcmVuY2gh",
            String::from_utf8(encode_base64(b"I am french!".to_vec())).unwrap()
        );
        assert_eq!(
            "YWJjZGVmZ2hp",
            String::from_utf8(encode_base64(b"abcdefghi".to_vec())).unwrap()
        );
        assert_eq!(
            "YWJjZGVmZ2hpag==",
            String::from_utf8(encode_base64(b"abcdefghij".to_vec())).unwrap()
        );
        assert_eq!(
            "YWJjZGVmZ2hpams=",
            String::from_utf8(encode_base64(b"abcdefghijk".to_vec())).unwrap()
        );
        assert_eq!(
            "YWJjZGVmZ2hpamts",
            String::from_utf8(encode_base64(b"abcdefghijkl".to_vec())).unwrap()
        );
    }

    #[test]
    fn decode() {
        assert_eq!(get_value_encoded(BASE64_MAP[5]).unwrap(), 5);
        assert_eq!(get_value_encoded(BASE64_MAP[15]).unwrap(), 15);
        assert_eq!(get_value_encoded(BASE64_MAP[25]).unwrap(), 25);
        assert_eq!(get_value_encoded(BASE64_MAP[53]).unwrap(), 53);
        assert_eq!(get_value_encoded(BASE64_MAP[62]).unwrap(), 62);
        assert_eq!(get_value_encoded(BASE64_MAP[63]).unwrap(), 63);
        assert_eq!(
            "abcdefghijkl",
            String::from_utf8(decode_base64(b"YWJjZGVmZ2hpamts".to_vec()).unwrap()).unwrap()
        );
        assert_eq!(
            "<div dir=\"ltr\">Hey émoji 😍</div>\r\n",
            String::from_utf8(
                decode_base64(b"PGRpdiBkaXI9Imx0ciI+SGV5IMOpbW9qaSDwn5iNPC9kaXY+DQo=".to_vec())
                    .unwrap()
            )
            .unwrap()
        );
    }
}
