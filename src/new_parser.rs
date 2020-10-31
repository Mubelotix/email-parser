#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Known(&'static str)
}

#[derive(Debug)]
pub enum String<'a> {
    Reference(&'a [u8]),
    Owned(std::string::String),
}

impl<'a> String<'a> {
    fn as_str(&self) -> &str {
        match self {
            String::Reference(string) => unsafe {
                // the parser is using only safe ASCII characters
                std::str::from_utf8_unchecked(string)
            },
            String::Owned(string) => &string
        }
    }

    fn into_owned(self) -> String<'static> {
        match self {
            String::Reference(_) => String::Owned(self.as_str().to_string()),
            String::Owned(string) => String::Owned(string)
        }
    }
}

impl<'a> std::ops::Add for String<'a> {
    type Output = String<'a>;

    fn add(self, rhs: String<'a>) -> Self::Output {
        match self {
            String::Reference(data1) => {
                if let String::Reference(data2) = rhs {
                    if let (Some(first1), Some(last1), Some(first2), Some(last2)) = (data1.first(), data1.last(), data2.first(), data2.last()) {
                        // if the two references are consecutive in memory, we create a third reference containing them
                        unsafe {
                            let first1 = first1 as *const u8;
                            let last1 = last1 as *const u8;
                            let first2 = first2 as *const u8;
                            let last2 = last2 as *const u8;
                            if last1 as usize + std::mem::size_of::<u8>() == first2 as usize { // this is what guarantee safety
                                let slice = std::slice::from_raw_parts(first1, last2 as usize - first1 as usize + 1);
                                return String::Reference(slice);
                            }
                        }
                    }
                }
                self.into_owned() + rhs.into_owned()
            },
            String::Owned(mut string) => {
                string.push_str(rhs.as_str());
                String::Owned(string)
            },
        }
    }
}

#[test]
fn unsafe_add_test() {
    let data = b"abcdef";
    let data1 = String::Reference(&data[..3]);
    let data2 = String::Reference(&data[3..]);

    let data3 = String::Reference(&data[..2]);
    let data4 = String::Reference(&data[3..]);

    assert!(matches!(data1+data2, String::Reference(_)));
    assert!(matches!(data3+data4, String::Owned(_)));
}

pub mod combinators {
    use super::Error;

    pub fn inc_while<F>(input: &[u8], idx: &mut usize, mut condition: F) where
        F: FnMut(u8) -> bool {
        while let Some(character) = input.get(*idx) {
            if condition(*character) {
                *idx += 1;
            } else {
                break;
            }
        }
    }

    pub fn inc_while1<F>(input: &[u8], idx: &mut usize, mut condition: F) -> Result<(), ()> where
        F: FnMut(u8) -> bool {
        match input.get(*idx) {
            Some(c) if condition(*c) => {
                *idx += 1;
            }
            _e => {
                return Err(());
            }
        };
        while let Some(character) = input.get(*idx) {
            if condition(*character) {
                *idx += 1;
            } else {
                break;
            }
        }
        Ok(())
    }

    pub fn take_while1<F>(input: &[u8], condition: F) -> Result<(&[u8], &[u8]), ()> where
    F: FnMut(u8) -> bool {
        let mut idx = 0;
        inc_while1(input, &mut idx, condition)?;
        Ok((&input[idx..], &input[..idx]))
    }

    pub fn inc_tag(input: &[u8], idx: &mut usize, tag: &[u8]) -> Result<(), ()> {
        if input[*idx..].starts_with(tag) {
            *idx += tag.len();
            Ok(())
        } else {
            Err(())
        }
    }

    #[allow(unused_must_use)]
    pub fn inc_after_opt<F, G>(input: &[u8], idx: &mut usize, mut optional_pattern: F, mut required_pattern: G) -> Result<(), Error> where
        F: FnMut(&[u8], &mut usize) -> Result<(), Error>,
        G: FnMut(&[u8], &mut usize) -> Result<(), Error> {
        let before = *idx;
        optional_pattern(input, idx);
        match required_pattern(input, idx) {
            Ok(()) => Ok(()),
            Err(e) => {
                *idx = before;
                Err(e)
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use super::super::character_groups::*;

        #[test]
        fn test_inc_while() {
            let mut idx = 0;
            let input = b"     aaa";
            inc_while(input, &mut idx, is_wsp);
            assert_eq!(idx, 5);
            assert_eq!(&input[..idx], b"     ");
            assert_eq!(&input[idx..], b"aaa");
        }
    }
}

pub mod character_groups {
    #[inline]
    pub fn is_wsp(character: u8) -> bool {
        character == 9 || character == 32
    }

    #[inline]
    pub fn is_ctext(character: u8) -> bool {
        (character >= 33 && character <= 39) ||
        (character >= 42 && character <= 91) ||
        (character >= 93 && character <= 126)
    }

    #[inline]
    pub fn is_vchar(character: u8) -> bool {
        character >= 0x21 && character <= 0x7e
    }

    #[inline]
    pub fn is_alpha(c: u8) -> bool {
        (c >= 0x41 && c <= 0x5a) ||
        (c >= 0x61 && c <= 0x7a)
    }

    #[inline]
    pub fn is_digit(c: u8) -> bool {
        c >= 0x30 && c <= 0x39
    }

    #[inline]
    pub fn is_atext(c: u8) -> bool {
        is_alpha(c) ||
        is_digit(c) ||
        c == b'!' ||
        c == b'#' ||
        c == b'$' ||
        c == b'%' ||
        c == b'&' ||
        c == b'\''||
        c == b'*' ||
        c == b'+' ||
        c == b'-' ||
        c == b'/' ||
        c == b'=' ||
        c == b'?' ||
        c == b'^' ||
        c == b'_' ||
        c == b'`' ||
        c == b'{' ||
        c == b'|' ||
        c == b'}' ||
        c == b'~'
    }

    #[inline]
    pub fn special(c: u8) -> bool {
        c == b'(' ||
        c == b')' ||
        c == b'<' ||
        c == b'>' ||
        c == b'[' ||
        c == b']' ||
        c == b':' ||
        c == b';' ||
        c == b'@' ||
        c == b'\\' ||
        c == b',' ||
        c == b'.' ||
        c == b'"'
    }

    #[inline]
    pub fn is_qtext(c: u8) -> bool {
        (c >= 35 && c <= 126 && c != 92) || c == 33
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
}

pub mod whitespaces {
    use super::*;
    use super::combinators::*;
    use super::character_groups::*;

    pub fn inc_fws(input: &[u8], mut idx: &mut usize) -> Result<(), Error> {
        let first_value = *idx;
        inc_while(input, &mut idx, is_wsp);
        if inc_tag(input, &mut idx, b"\r\n").is_err() {
            *idx = first_value;
        }
        if inc_while1(input, &mut idx, is_wsp).is_err() {
            return Err(Error::Known("Missing whitespaces in a folding whitespace"));
        }
        Ok(())
    }
    
    pub fn take_fws(input: &[u8]) -> Result<(&[u8], &[u8]), Error> {
        let mut idx = 0;
        inc_fws(input, &mut idx)?;
    
        Ok((&input[idx..], &input[..idx]))
    }
    
    fn inc_ccontent(input: &[u8], idx: &mut usize) -> Result<(), Error> {
        if inc_while1(input, idx, is_ctext).is_ok() || inc_quoted_pair(input, idx).is_ok() || inc_comment(input, idx).is_ok() {
            Ok(())
        } else {
            Err(Error::Known("Invalid ccontent"))
        }
    }
    
    pub fn inc_comment(input: &[u8], idx: &mut usize) -> Result<(), Error> {
        if inc_tag(input, idx, b"(").is_err() {
            return Err(Error::Known("Comment is expected to start with a '('."));
        }
        
        while inc_after_opt(input, idx, inc_fws, inc_ccontent).is_ok() { }
    
        let _ = inc_fws(input, idx);
        if inc_tag(input, idx, b")").is_err() {
            return Err(Error::Known("Comment is expected to end with a ')'."));
        }
    
        Ok(())
    }
    
    pub fn inc_cfws(input: &[u8], idx: &mut usize) -> Result<(), Error> {
        if inc_after_opt(input, idx, inc_fws, inc_comment).is_ok() {
            while inc_after_opt(input, idx, inc_fws, inc_comment).is_ok() {}
            //let _ = inc_fws(input, idx);
            inc_fws(input, idx)
        } else {
            inc_fws(input, idx)
        }
    }

    pub fn take_cfws(input: &[u8]) -> Result<(&[u8], &[u8]), Error> {
        let mut idx = 0;
        inc_cfws(input, &mut idx)?;
        Ok((&input[idx..], &input[..idx]))
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_fws() {
            assert_eq!(take_fws(b"   test").unwrap().1, b"   ");
            assert_eq!(take_fws(b"   \r\n  test").unwrap().1, b"   \r\n  ");
        
            assert!(take_fws(b"  \r\ntest").is_err());
            assert!(take_fws(b"\r\ntest").is_err());
            assert!(take_fws(b"test").is_err());
        }

        #[test]
        fn test_inc_ccontent() {
            let mut idx = 0;
            inc_ccontent(b"abcde", &mut idx).unwrap();
            assert_eq!(idx, 5);
        
            let mut idx = 0;
            inc_ccontent(b"ab)cde", &mut idx).unwrap();
            assert_eq!(idx, 2);
        }

        #[test]
        fn test_inc_comment() {
            let mut idx = 0;
            inc_comment(b"(this is a comment)", &mut idx).unwrap();
            assert_eq!(idx, 19);
            
            let mut idx = 0;
            inc_comment(b"(a comment) and a value", &mut idx).unwrap();
            assert_eq!(idx, 11);
        
            let mut idx = 0;
            inc_comment(b"(this is a comment (and another comment)) and a value", &mut idx).unwrap();
            assert_eq!(idx, 41);
        
            assert!(inc_comment(b"a value", &mut 0).is_err());
            assert!(inc_comment(b"(unclosed comment", &mut 0).is_err());
        }

        #[test]
        fn test_inc_cfws() {
            let mut idx = 0;
            inc_cfws(b"  (this is a comment)\r\n (this is a second comment)  value", &mut idx).unwrap();
            assert_eq!(idx, 52);
        
            idx = 0;
            inc_cfws(b"  (this is a comment)\r\n (this is a second comment)\r\n  value", &mut idx).unwrap();
            assert_eq!(idx, 54);
        }
    }
}

use character_groups::*;
use combinators::*;
use whitespaces::*;

pub fn inc_quoted_pair(input: &[u8], idx: &mut usize) -> Result<(), Error> {
    if input[*idx..].starts_with(b"\\") {
        if let Some(character) = input.get(*idx + 1) {
            if is_vchar(*character) || is_wsp(*character) {
                *idx += 2;
                Ok(())
            } else {
                Err(Error::Known("The quoted-pair character is no a vchar or a wsp."))
            }
        } else {
            Err(Error::Known("The quoted-pair has no second character."))
        }
    } else {
        Err(Error::Known("The quoted-pair does not start with a '\\'."))
    }
}

pub fn take_quoted_pair(input: &[u8]) -> Result<char, Error> {
    if input.starts_with(b"\\") {
        if let Some(character) = input.get(1) {
            if is_vchar(*character) || is_wsp(*character) {
                Ok(*character as char)
            } else {
                Err(Error::Known("The quoted-pair character is no a vchar or a wsp."))
            }
        } else {
            Err(Error::Known("The quoted-pair has no second character."))
        }
    } else {
        Err(Error::Known("The quoted-pair does not start with a '\\'."))
    }
}

pub fn take_quoted_string(input: &[u8]) -> Result<String, Error> {
    let input = if let Ok((input, cfws)) = take_cfws(input) {
        input
    } else {
        input
    };
    let input = if input.starts_with(b"\"") {
        &input[1..]
    } else {
        return Err(Error::Known("Quoted string must begin with a dquote"));
    };

    loop {

    }
    let input = if input.starts_with(b"\"") {
        &input[1..]
    } else {
        return Err(Error::Known("Quoted string must end with a dquote"));
    };
    let input = if let Ok((input, cfws)) = take_cfws(input) {
        input
    } else {
        input
    };
}

pub fn inc_atom(input: &[u8], idx: &mut usize) -> Result<(), Error> {
    let _ = inc_cfws(input, idx);
    if inc_while1(input, idx, is_atext).is_err() {
        return Err(Error::Known("No atom here"));
    }
    let _ = inc_cfws(input, idx);
    Ok(())
}

pub fn inc_dot_atom_text(input: &[u8], idx: &mut usize) -> Result<(), Error> {
    if inc_while1(input, idx, is_atext).is_err() {
        return Err(Error::Known("Expected atom character at the beggining of a dot_atom_text"));
    }

    loop {
        let idx_before = *idx;
        if inc_tag(input, idx, b".").is_err() {
            break;
        }
        if inc_while1(input, idx, is_atext).is_err() {
            *idx = idx_before;
            break;
        }
    }

    Ok(())
}

pub fn inc_dot_atom(input: &[u8], idx: &mut usize) -> Result<(), Error> {
    let _ = inc_cfws(input, idx);
    inc_dot_atom_text(input, idx)?;
    let _ = inc_cfws(input, idx);

    Ok(())
}

pub fn take_atom(mut input: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    let (mut input, atom) = take_while1(input, is_atext).map_err(|_| Error::Known("Atom required"))?;
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    Ok((input, atom))
}

pub fn take_dot_atom_text(input: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    let mut idx = 0;
    inc_dot_atom_text(input, &mut idx)?;
    Ok((&input[idx..], &input[..idx]))
}

pub fn take_dot_atom(mut input: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    let (mut input, dot_atom) = take_dot_atom_text(input)?;
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    Ok((input, dot_atom))
}

#[test]
fn test_quoted_pair() {
    assert!(inc_quoted_pair(b"\\rtest", &mut 0).is_ok());
    assert!(inc_quoted_pair(b"\\ test", &mut 0).is_ok());

    assert_eq!(take_quoted_pair(b"\\rtest"), Ok('r'));
    assert_eq!(take_quoted_pair(b"\\ test"), Ok(' '));

    assert!(inc_quoted_pair(b"\\", &mut 0).is_err());
    assert!(inc_quoted_pair(b"\\\0", &mut 0).is_err());
    assert!(inc_quoted_pair(b"test", &mut 0).is_err());
}

#[test]
fn test_atom() {
    fn strize<'a, T: std::fmt::Debug>(i: Result<(&'a [u8], &'a [u8]), T>) -> &'a str {
        std::str::from_utf8(i.unwrap().1).unwrap()
    }

    assert_eq!(strize(take_atom(b"this is a test")), "this");
    assert_eq!(strize(take_atom(b"   averylongatom ")), "averylongatom");
    assert_eq!(strize(take_dot_atom_text(b"this.is.a.test")), "this.is.a.test");
    assert_eq!(strize(take_dot_atom(b"  this.is.a.test ")), "this.is.a.test");
}