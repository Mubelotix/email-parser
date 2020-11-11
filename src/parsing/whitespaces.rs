use crate::prelude::*;
use std::borrow::Cow;

#[inline]
pub fn fws(input: &[u8]) -> Res<Cow<str>> {
    let (input, before) = optional(input, |input| {
        pair(
            input,
            |input| take_while(input, is_wsp),
            |input| tag(input, b"\r\n"),
        )
    });
    let (input, after) = take_while1(input, is_wsp)?;

    if let Some((mut before, _crlf)) = before {
        add_string(&mut before, after);
        Ok((input, before))
    } else {
        Ok((input, after))
    }
}

#[inline]
pub fn ccontent(input: &[u8]) -> Res<Cow<str>> {
    match_parsers(
        input,
        &mut [
            (|i| take_while1(i, is_ctext)) as fn(input: &[u8]) -> Res<Cow<str>>,
            quoted_pair,
            comment,
        ][..],
    )
}

#[inline]
pub fn comment(input: &[u8]) -> Res<Cow<str>> {
    let (input, ()) = tag(input, b"(")?;

    let (input, _) = ignore_many(input, |input| {
        pair(input, |i| Ok(optional(i, fws)), ccontent)
    })?;

    let (input, _) = optional(input, fws);
    let (input, ()) = tag(input, b")")?;

    Ok((input, empty_string()))
}

#[inline]
pub fn cfws(input: &[u8]) -> Res<Cow<str>> {
    fn real_cfws(mut input: &[u8]) -> Res<Cow<str>> {
        let mut output = empty_string();

        let (new_input, folding_wsp) = optional(input, fws);
        if let Ok((new_input, _comment)) = comment(new_input) {
            input = new_input;
            if let Some(s) = folding_wsp {
                add_string(&mut output, s);
            }
        } else {
            return Err(Error::Known("Expected at least one comment"));
        }

        loop {
            let (new_input, folding_wsp) = optional(input, fws);

            if let Ok((new_input, _comment)) = comment(new_input) {
                input = new_input;
                if let Some(s) = folding_wsp {
                    add_string(&mut output, s);
                }
            } else {
                break;
            }
        }

        let (input, fws) = optional(input, fws);
        if let Some(s) = fws {
            add_string(&mut output, s);
        }

        Ok((input, output))
    }

    match_parsers(input, &mut [real_cfws, fws][..])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fws() {
        assert_eq!(fws(b"   test").unwrap().1, "   ");
        assert_eq!(fws(b" test").unwrap().1, " ");
        assert_eq!(fws(b"   \r\n  test").unwrap().1, "     ");

        assert!(fws(b"  \r\ntest").is_err());
        assert!(fws(b"\r\ntest").is_err());
        assert!(fws(b"test").is_err());
    }

    #[test]
    fn test_ccontent() {
        assert_eq!(ccontent(b"abcde").unwrap().1, "abcde");
        assert_eq!(ccontent(b"ab)cde").unwrap().1, "ab");
    }

    #[test]
    fn test_comment() {
        assert_eq!(comment(b"(this is a comment)").unwrap().0.len(), 0);
        assert_eq!(comment(b"(a comment) and a value").unwrap().0.len(), 12);
        assert_eq!(
            comment(b"(this is a comment (and another comment)) and a value")
                .unwrap()
                .0
                .len(),
            12
        );

        assert!(comment(b"a value").is_err());
        assert!(comment(b"(unclosed comment").is_err());
    }

    #[test]
    fn test_cfws() {
        assert_eq!(
            cfws(b"  (this is a comment)\r\n (this is a second comment)  value")
                .unwrap()
                .1,
            "     "
        );

        assert_eq!(
            cfws(b"  (this is a comment)\r\n (this is a second comment)\r\n  value")
                .unwrap()
                .1,
            "     "
        );
    }
}
