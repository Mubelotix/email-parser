#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Known(&'static str)
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
    
    pub fn inc_quoted_pair(input: &[u8], idx: &mut usize) -> Result<(), Error> {
        if input[*idx..].starts_with(b"\\") {
            if let Some(character) = input.get(*idx + 1) {
                if is_vchar(*character) || is_wsp(*character) {
                    *idx += 2;
                    return Ok(());
                } else {
                    return Err(Error::Known("The quoted-pair character is no a vchar or a wsp."));
                }
            } else {
                return Err(Error::Known("The quoted-pair has no second character."));
            }
        } else {
            return Err(Error::Known("The quoted-pair does not start with a '\\'."))
        }
    }
    
    fn inc_ccontent(input: &[u8], idx: &mut usize) -> Result<(), Error> {
        if inc_while1(input, idx, is_ctext).is_ok() {
            Ok(())
        } else if inc_quoted_pair(input, idx).is_ok() {
            Ok(())
        } else if inc_comment(input, idx).is_ok() {
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
        fn test_inc_quoted_pair() {
            assert!(inc_quoted_pair(b"\\rtest", &mut 0).is_ok());
            assert!(inc_quoted_pair(b"\\ test", &mut 0).is_ok());
        
            assert!(inc_quoted_pair(b"\\", &mut 0).is_err());
            assert!(inc_quoted_pair(b"\\\0", &mut 0).is_err());
            assert!(inc_quoted_pair(b"test", &mut 0).is_err());
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
