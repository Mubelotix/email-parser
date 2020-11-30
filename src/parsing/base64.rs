const BASE64_MAP: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];

pub fn encode_base64(data: Vec<u8>) -> Vec<u8> {
    let mut encoded_data = Vec::new();
    let mut bytes = data.iter();

    while let Some(byte1) = bytes.next() {
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
    }

    encoded_data
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
}
