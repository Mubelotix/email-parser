use crate::prelude::Res;

use crate::prelude::*;
use std::borrow::Cow;

use super::{base64, quoted_printables};

#[inline]
fn especials(c: u8) -> bool {
    c == b'('
        || c == b')'
        || c == b'<'
        || c == b'>'
        || c == b'@'
        || c == b','
        || c == b';'
        || c == b':'
        || c == b'\\'
        || c == b'"'
        || c == b'/'
        || c == b'['
        || c == b']'
        || c == b'?'
        || c == b'.'
        || c == b'='
}

fn charset(input: &[u8]) -> Res<Cow<str>> {
    let (input, mut charset) = take_while1(input, |c| c > 0x20 && c < 0x7F && !especials(c))?;
    let mut change_needed = false;
    for c in charset.chars() {
        if c.is_uppercase() {
            change_needed = true;
        }
    }
    if change_needed {
        charset = Cow::Owned(charset.to_ascii_lowercase());
    }
    Ok((input, charset))
}

fn encoding(input: &[u8]) -> Res<Cow<str>> {
    let (input, mut encoding) = take_while1(input, |c| c > 0x20 && c < 0x7F && !especials(c))?;
    let mut change_needed = false;
    for c in encoding.chars() {
        if c.is_lowercase() {
            change_needed = true;
        }
    }
    if change_needed {
        encoding = Cow::Owned(encoding.to_ascii_uppercase());
    }
    Ok((input, encoding))
}

fn encoded_text(input: &[u8]) -> Res<Cow<str>> {
    take_while1(input, |c| c > 0x20 && c <= 0x7E && c != b'?')
}

fn encoded_word(input: &[u8]) -> Res<Cow<str>> {
    let (input, _) = tag(input, b"=?")?;
    let (input, charset) = charset(input)?;
    let (input, _) = tag(input, b"?")?;
    let (input, encoding) = encoding(input)?;
    let (input, _) = tag(input, b"?")?;
    let (input, data) = encoded_text(input)?;
    let (input, _) = tag(input, b"?=")?;

    let value = match encoding.as_ref() {
        "B" => base64::decode_base64(data.into_owned().into_bytes())?,
        "Q" => quoted_printables::decode_header_qp(data.into_owned().into_bytes()),
        _ => return Err(Error::Known("Unknown encoding")),
    };

    use textcode::*;
    let text: Cow<str> = match charset.as_ref() {
        "utf-8" | "us-ascii" => Cow::Owned(
            String::from_utf8(value).map_err(|_| Error::Known("Invalid text encoding"))?,
        ),
        "iso-8859-1" => Cow::Owned(iso8859_1::decode_to_string(&value)),
        "iso-8859-2" => Cow::Owned(iso8859_2::decode_to_string(&value)),
        "iso-8859-3" => Cow::Owned(iso8859_3::decode_to_string(&value)),
        "iso-8859-4" => Cow::Owned(iso8859_4::decode_to_string(&value)),
        "iso-8859-5" => Cow::Owned(iso8859_5::decode_to_string(&value)),
        "iso-8859-6" => Cow::Owned(iso8859_6::decode_to_string(&value)),
        "iso-8859-7" => Cow::Owned(iso8859_7::decode_to_string(&value)),
        "iso-8859-8" => Cow::Owned(iso8859_8::decode_to_string(&value)),
        "iso-8859-9" => Cow::Owned(iso8859_9::decode_to_string(&value)),
        "iso-8859-10" => Cow::Owned(iso8859_10::decode_to_string(&value)),
        "iso-8859-11" => Cow::Owned(iso8859_11::decode_to_string(&value)),
        "iso-8859-13" => Cow::Owned(iso8859_13::decode_to_string(&value)),
        "iso-8859-14" => Cow::Owned(iso8859_14::decode_to_string(&value)),
        "iso-8859-15" => Cow::Owned(iso8859_15::decode_to_string(&value)),
        "iso-8859-16" => Cow::Owned(iso8859_16::decode_to_string(&value)),
        "iso-6937" => Cow::Owned(iso6937::decode_to_string(&value)),
        "gb2312" => Cow::Owned(gb2312::decode_to_string(&value)),
        _ => return Err(Error::Known("Unknown charset")),
    };

    Ok((input, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_word_test() {
        assert_eq!("this is some text", encoded_word(b"=?iso-8859-1?q?this=20is=20some=20text?=").unwrap().1);
        assert_eq!("Don't forget! Claim your $5 today 💸", encoded_word(b"=?utf-8?q?Don=27t_forget!_Claim_your_=245_today_=F0=9F=92=B8?=").unwrap().1);
        assert_eq!("Chloé Helloco", encoded_word(b"=?UTF-8?Q?Chlo=C3=A9_Helloco?=").unwrap().1);

        assert_eq!("If you can read this yo", encoded_word(b"=?ISO-8859-1?B?SWYgeW91IGNhbiByZWFkIHRoaXMgeW8=?=").unwrap().1);
        assert_eq!("u understand the example.", encoded_word(b"=?ISO-8859-2?B?dSB1bmRlcnN0YW5kIHRoZSBleGFtcGxlLg==?=").unwrap().1);
    }
}