use crate::prelude::*;

pub fn take_fws(input: &[u8]) -> Res<String> {
    let (input, before) = optional(input, |input| {
        take_pair(
            input,
            |input| take_while(input, is_wsp),
            |input| tag(input, b"\r\n"),
        )
    });
    let (input, after) = take_while1(input, is_wsp)?;

    if let Some((before, _crlf)) = before {
        Ok((input, before + after))
    } else {
        Ok((input, after))
    }
}

pub fn take_ccontent(input: &[u8]) -> Res<String> {
    match_parsers(
        input,
        &mut [
            (|i| take_while1(i, is_ctext)) as fn(input: &[u8]) -> Res<String>,
            take_quoted_pair,
            take_comment,
        ][..],
    )
}

pub fn take_comment(input: &[u8]) -> Res<String> {
    let (input, ()) = tag(input, b"(")?;

    let (input, _) = ignore_many(input, |input| take_pair(
        input,
        |i| Ok(optional(i, take_fws)),
        take_ccontent,
    ))?;

    let (input, _) = optional(input, take_fws);
    let (input, ()) = tag(input, b")")?;


    Ok((input, String::new()))
}

pub fn take_cfws(input: &[u8]) -> Res<String> {
    fn take_real_cfws(mut input: &[u8]) -> Res<String> {
        let mut output = String::new();

        let (new_input, fws) = optional(input, take_fws);
        if let Ok((new_input, _comment)) = take_comment(new_input) {
            input = new_input;
            if let Some(s) = fws {
                output += s;
            }
        } else {
            return Err(Error::Known("Expected at least one comment"))
        }

        loop {
            let (new_input, fws) = optional(input, take_fws);

            if let Ok((new_input, _comment)) = take_comment(new_input) {
                input = new_input;
                if let Some(s) = fws {
                    output += s;
                }
            } else {
                break;
            }
        }

        let (input, fws) = optional(input, take_fws);
        if let Some(s) = fws {
            output += s;
        }

        Ok((input, output))
    }
    
    match_parsers(input, &mut [take_real_cfws, take_fws][..])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fws() {
        assert_eq!(take_fws(b"   test").unwrap().1, "   ");
        assert_eq!(take_fws(b" test").unwrap().1, " ");
        assert_eq!(take_fws(b"   \r\n  test").unwrap().1, "     ");

        assert!(take_fws(b"  \r\ntest").is_err());
        assert!(take_fws(b"\r\ntest").is_err());
        assert!(take_fws(b"test").is_err());
    }

    #[test]
    fn test_ccontent() {
        assert_eq!(take_ccontent(b"abcde").unwrap().1, "abcde");
        assert_eq!(take_ccontent(b"ab)cde").unwrap().1, "ab");
    }

    #[test]
    fn test_comment() {
        assert_eq!(take_comment(b"(this is a comment)").unwrap().0.len(), 0);
        assert_eq!(take_comment(b"(a comment) and a value").unwrap().0.len(), 12);
        assert_eq!(take_comment(b"(this is a comment (and another comment)) and a value").unwrap().0.len(), 12);

        assert!(take_comment(b"a value").is_err());
        assert!(take_comment(b"(unclosed comment").is_err());
    }

    #[test]
    fn test_cfws() {
        assert_eq!(take_cfws(
            b"  (this is a comment)\r\n (this is a second comment)  value"
        )
        .unwrap().1, "     ");

        assert_eq!(take_cfws(
            b"  (this is a comment)\r\n (this is a second comment)\r\n  value"
        )
        .unwrap().1, "     ");
    }
}
    