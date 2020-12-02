use crate::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

fn before_boundary<'a, 'b>(input: &'a [u8], boundary: &'b [u8]) -> Res<'a, &'a [u8]> {
    let full_boundary_len = 2 + 2 + boundary.len() + 2;

    unsafe {
        // FIXME: ignore whitespaces after the boundary
        if input.get_unchecked(..2) == b"--"
            && input.get_unchecked(2..2 + boundary.len()) == boundary
            && input.get_unchecked(2 + boundary.len()..full_boundary_len - 2) == b"\r\n"
        {
            return Ok((
                input.get_unchecked(full_boundary_len - 2..),
                b"",
            ));
        }
    }

    for idx in 0..input.len().saturating_sub(full_boundary_len) + 1 {
        unsafe {
            // FIXME: ignore whitespaces after the boundary
            if input.get_unchecked(idx..idx + 4) == b"\r\n--"
                && input.get_unchecked(idx + 4..idx + 4 + boundary.len()) == boundary
                && input.get_unchecked(idx + 4 + boundary.len()..idx + full_boundary_len) == b"\r\n"
            {
                return Ok((
                    input.get_unchecked(idx + full_boundary_len..),
                    input.get_unchecked(..idx),
                ));
            }
        }
    }

    Err(Error::Known("boundary not found"))
}

fn before_closing_boundary<'a, 'b>(input: &'a [u8], boundary: &'b [u8]) -> Res<'a, &'a [u8]> {
    let full_boundary_len = 2 + 2 + boundary.len() + 2 + 2;
    for idx in 0..input.len().saturating_sub(full_boundary_len) + 1 {
        unsafe {
            // FIXME: ignore whitespaces after the boundary
            if input.get_unchecked(idx..idx + 4) == b"\r\n--"
                && input.get_unchecked(idx + 4..idx + 4 + boundary.len()) == boundary
                && input.get_unchecked(idx + 4 + boundary.len()..idx + full_boundary_len)
                    == b"--\r\n"
            {
                return Ok((
                    input.get_unchecked(idx + full_boundary_len..),
                    input.get_unchecked(..idx),
                ));
            }
        }
    }

    Err(Error::Known("closing boundary not found"))
}

pub fn parse_multipart<'a>(input: &'a [u8], parameters: HashMap<Cow<str>, Cow<str>>) -> Result<Vec<Entity<'a>>, Error> {
    let boundary = parameters.get("boundary").ok_or(Error::Known("Missing boundary parameter"))?;
    let (input, mut parts) = many(&input, |i| before_boundary(i, boundary.as_bytes()))?;
    let (_epilogue, last_part) = before_closing_boundary(input, boundary.as_bytes())?;
    parts.push(last_part);
    parts.remove(0); // the prelude


    let mut entities = Vec::new();
    for part in parts {
        entities.push(super::entity::entity(Cow::Borrowed(part))?);
    }

    Ok(entities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipart() {
        parse_multipart(
            b"This is the preamble.  It is to be ignored, though it\r\nis a handy place for composition agents to include an\r\nexplanatory note to non-MIME conformant readers.\r\n\r\n--simple boundary\r\n\r\nThis is implicitly typed plain US-ASCII text.\r\nIt does NOT end with a linebreak.\r\n--simple boundary\r\nContent-type: text/plain; charset=us-ascii\r\n\r\nThis is explicitly typed plain US-ASCII text.\r\nIt DOES end with a linebreak.\r\n\r\n--simple boundary--\r\n\r\nThis is the epilogue.  It is also to be ignored.",
            vec![(Cow::Borrowed("boundary"), Cow::Borrowed("simple boundary"))].into_iter().collect(),
        )
        .unwrap();
    }

    #[test]
    fn test_boundary() {
        assert_eq!(
            b"",
            before_boundary(
                b"--boundary\r\nI am making a not here: huge success",
                b"boundary"
            )
            .unwrap()
            .1
        );
        assert_eq!(
            b"This was a triumph",
            before_boundary(
                b"This was a triumph\r\n--boundary\r\nI am making a not here: huge success",
                b"boundary"
            )
            .unwrap()
            .1
        );
        assert_eq!(
            b"",
            before_boundary(
                b"\r\n--boundary\r\nI am making a not here: huge success",
                b"boundary"
            )
            .unwrap()
            .1
        );
        assert_eq!(
            b"I am making a not here: huge success",
            before_boundary(
                b"This was a triumph\r\n--boundary\r\nI am making a not here: huge success",
                b"boundary"
            )
            .unwrap()
            .0
        );
        assert_eq!(
            b"I am making a not here: huge success",
            before_closing_boundary(
                b"This was a triumph\r\n--boundary--\r\nI am making a not here: huge success",
                b"boundary"
            )
            .unwrap()
            .0
        );
    }
}
