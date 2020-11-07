use crate::prelude::*;
use crate::parsing::fields::{Field, take_fields};

pub fn take_line(input: &[u8]) -> Res<String> {
    let max_idx = std::cmp::min(input.len(), 998);

    // index cannot be out of range so no need to check
    unsafe {
        for i in 0..max_idx {
            if !is_text(*input.get_unchecked(i)) {
                return Ok((
                    input.get_unchecked(i..),
                    String::Reference(input.get_unchecked(..i)),
                ));
            }
        }

        Ok((
            input.get_unchecked(max_idx..),
            String::Reference(input.get_unchecked(..max_idx)),
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

pub fn take_body_lines(input: &[u8]) -> Result<Vec<String>, Error> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    let (mut input, ()) = tag(input, b"\r\n")?;

    let mut lines = Vec::new();
    loop {
        let (new_input, new_line) = take_line(input)?;
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

pub fn take_body(input: &[u8]) -> Result<Option<String>, Error> {
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
        String::Reference(input.get_unchecked(2..))
    }))
}

pub fn parse_message(input: &[u8]) -> Result<(Vec<Field>, Option<String>), Error> {
    let (input, fields) = take_fields(input)?;
    let body = take_body(input)?;
    
    Ok((fields, body))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_body() {
        assert_eq!(
            take_line(b"This is a line\r\nAnd this is a second line")
                .unwrap()
                .1,
            "This is a line"
        );
        assert_eq!(
            take_body_lines(b"\r\nThis is a line\r\nAnd this is a second line")
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            take_body(b"\r\nThis is a line\r\nAnd this is a second line")
                .unwrap()
                .unwrap(),
            "This is a line\r\nAnd this is a second line"
        );
    }

    #[test]
    fn test_full_message() {
        println!("{:#?}", parse_message(include_bytes!("../../mail.txt")).unwrap());
    }
}
