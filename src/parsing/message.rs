use crate::parsing::fields::{fields, Field};
use crate::prelude::*;
use std::borrow::Cow;

pub fn line(input: &[u8]) -> Res<Cow<str>> {
    let max_idx = std::cmp::min(input.len(), 998);

    // index cannot be out of range so no need to check
    unsafe {
        for i in 0..max_idx {
            if !is_text(*input.get_unchecked(i)) {
                return Ok((
                    input.get_unchecked(i..),
                    from_slice(input.get_unchecked(..i)),
                ));
            }
        }

        Ok((
            input.get_unchecked(max_idx..),
            from_slice(input.get_unchecked(..max_idx)),
        ))
    }
}

pub fn check_line(input: &[u8]) -> Res<()> {
    let max_idx = std::cmp::min(input.len(), 998);

    // index cannot be out of range so no need to check
    unsafe {
        for i in 0..max_idx {
            if !is_text(*input.get_unchecked(i)) {
                return Ok((input.get_unchecked(i..), ()));
            }
        }

        Ok((input.get_unchecked(max_idx..), ()))
    }
}

pub fn body_lines(input: &[u8]) -> Result<Vec<Cow<str>>, Error> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    let (mut input, ()) = tag(input, b"\r\n")?;

    let mut lines = Vec::new();
    loop {
        let (new_input, new_line) = line(input)?;
        match tag(new_input, b"\r\n") {
            Ok((new_input, ())) => input = new_input,
            Err(e) => {
                if new_input.is_empty() {
                    lines.push(new_line);
                    break;
                } else {
                    return Err(e);
                }
            }
        }
        lines.push(new_line);
    }

    Ok(lines)
}

pub fn body(input: &[u8]) -> Result<Option<Cow<str>>, Error> {
    if input.is_empty() {
        return Ok(None);
    }

    let (mut new_input, ()) = tag(input, b"\r\n")?;

    loop {
        let (new_input2, ()) = check_line(new_input)?;
        match tag(new_input2, b"\r\n") {
            Ok((new_input2, ())) => new_input = new_input2,
            Err(e) => {
                if new_input2.is_empty() {
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }

    Ok(Some(unsafe {
        // there is a least 2 characters
        from_slice(input.get_unchecked(2..))
    }))
}

#[cfg(not(feature = "mime"))]
pub fn parse_message(input: &[u8]) -> Result<(Vec<Field>, Option<Cow<str>>), Error> {
    let (input, fields) = fields(input)?;
    let body = body(input)?;

    Ok((fields, body))
}

#[cfg(feature = "mime")]
pub fn parse_message(input: &[u8]) -> Result<(Vec<Field>, Option<&[u8]>), Error> {
    let (input, fields) = fields(input)?;

    if input.is_empty() {
        return Ok((fields, None));
    }

    let (new_input, ()) = tag(input, b"\r\n")?;

    Ok((fields, Some(new_input)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_body() {
        assert_eq!(
            line(b"This is a line\r\nAnd this is a second line")
                .unwrap()
                .1,
            "This is a line"
        );
        assert_eq!(
            body_lines(b"\r\nThis is a line\r\nAnd this is a second line")
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            body(b"\r\nThis is a line\r\nAnd this is a second line")
                .unwrap()
                .unwrap(),
            "This is a line\r\nAnd this is a second line"
        );
    }

    #[test]
    fn test_full_message() {
        //println!("{:#?}", parse_message(include_bytes!("../../mail.txt")).unwrap());
    }
}
