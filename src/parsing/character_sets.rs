use crate::prelude::*;

#[inline]
pub fn is_wsp(character: u8) -> bool {
    character == 9 || character == 32
}

#[inline]
pub fn is_ctext(character: u8) -> bool {
    (character >= 33 && character <= 39)
        || (character >= 42 && character <= 91)
        || (character >= 93 && character <= 126)
}

#[inline]
pub fn is_vchar(character: u8) -> bool {
    character >= 0x21 && character <= 0x7e
}

#[inline]
pub fn is_alpha(c: u8) -> bool {
    (c >= 0x41 && c <= 0x5a) || (c >= 0x61 && c <= 0x7a)
}

#[inline]
pub fn is_digit(c: u8) -> bool {
    c >= 0x30 && c <= 0x39
}

pub fn digit(input: &[u8]) -> Result<(&[u8], u8), Error> {
    match input.get(0) {
        Some(b'0') => Ok((&input[1..], 0)),
        Some(b'1') => Ok((&input[1..], 1)),
        Some(b'2') => Ok((&input[1..], 2)),
        Some(b'3') => Ok((&input[1..], 3)),
        Some(b'4') => Ok((&input[1..], 4)),
        Some(b'5') => Ok((&input[1..], 5)),
        Some(b'6') => Ok((&input[1..], 6)),
        Some(b'7') => Ok((&input[1..], 7)),
        Some(b'8') => Ok((&input[1..], 8)),
        Some(b'9') => Ok((&input[1..], 9)),
        _ => Err(Error::Unknown("Invalid digit")),
    }
}

pub fn two_digits(input: &[u8]) -> Result<(&[u8], u8), Error> {
    let (input, first) = digit(input)?;
    let (input, second) = digit(input)?;

    Ok((input, first * 10 + second))
}

#[inline]
pub fn is_dtext(c: u8) -> bool {
    (c >= 33 && c <= 90) || (c >= 94 && c <= 126)
}

#[inline]
pub fn is_atext(c: u8) -> bool {
    is_alpha(c)
        || is_digit(c)
        || c == b'!'
        || c == b'#'
        || c == b'$'
        || c == b'%'
        || c == b'&'
        || c == b'\''
        || c == b'*'
        || c == b'+'
        || c == b'-'
        || c == b'/'
        || c == b'='
        || c == b'?'
        || c == b'^'
        || c == b'_'
        || c == b'`'
        || c == b'{'
        || c == b'|'
        || c == b'}'
        || c == b'~'
}

#[inline]
pub fn special(c: u8) -> bool {
    c == b'('
        || c == b')'
        || c == b'<'
        || c == b'>'
        || c == b'['
        || c == b']'
        || c == b':'
        || c == b';'
        || c == b'@'
        || c == b'\\'
        || c == b','
        || c == b'.'
        || c == b'"'
}

#[inline]
pub fn tspecial(c: u8) -> bool {
    c == b'('
        || c == b')'
        || c == b'<'
        || c == b'>'
        || c == b'['
        || c == b']'
        || c == b':'
        || c == b';'
        || c == b'@'
        || c == b'\\'
        || c == b','
        || c == b'/'
        || c == b'?'
        || c == b'='
        || c == b'"'
}

#[inline]
pub fn is_qtext(c: u8) -> bool {
    (c >= 35 && c <= 126 && c != 92) || c == 33
}

#[inline]
pub fn is_ftext(c: u8) -> bool {
    (c >= 33 && c <= 57) || (c >= 59 && c <= 126)
}

#[inline]
pub fn is_text(c: u8) -> bool {
    c >= 1 && c <= 127 && c != 10 && c != 13
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wsp() {
        assert!(is_wsp(b' '));
        assert!(is_wsp(b'\t'));
        assert!(!is_wsp(b'a'));
        assert!(!is_wsp(b'e'));
    }
}
