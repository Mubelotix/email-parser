use crate::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

fn before_boundary_idx(input: &[u8], boundary: &[u8]) -> Result<(usize, usize), Error> {
    let full_boundary_len = 2 + 2 + boundary.len() + 2;

    // FIXME: ignore whitespaces after the boundary
    if full_boundary_len - 2 <= input.len()
        && input.get(..2) == Some(b"--")
        && input.get(2..2 + boundary.len()) == Some(boundary)
        && input.get(2 + boundary.len()..full_boundary_len - 2) == Some(b"\r\n")
    {
        return Ok((0, full_boundary_len - 2));
    }

    for idx in 0..input.len() {
        // FIXME: ignore whitespaces after the boundary
        if input.get(idx..idx + 4) == Some(b"\r\n--")
            && input.get(idx + 4..idx + 4 + boundary.len()) == Some(boundary)
            && input.get(idx + 4 + boundary.len()..idx + full_boundary_len) == Some(b"\r\n")
        {
            return Ok((idx, full_boundary_len));
        }
    }

    Err(Error::Known("boundary not found"))
}

fn before_boundary<'a, 'b>(input: &'a [u8], boundary: &'b [u8]) -> Res<'a, &'a [u8]> {
    let (before, len) = before_boundary_idx(input, boundary)?;

    // JUSTIFICATION
    //  Benefit
    //      Improve performances by avoiding checking the range.
    //  Correctness
    //      The function cannot return out of range indexes.
    unsafe {
        Ok((
            input.get_unchecked(before + len..),
            input.get_unchecked(..before),
        ))
    }
}

fn before_closing_boundary_idx(input: &[u8], boundary: &[u8]) -> Result<(usize, usize), Error> {
    let full_boundary_len = 2 + 2 + boundary.len() + 2 + 2;
    for idx in 0..input.len() {
        // FIXME: ignore whitespaces after the boundary
        if input.get(idx..idx + 4) == Some(b"\r\n--")
            && input.get(idx + 4..idx + 4 + boundary.len()) == Some(boundary)
            && input.get(idx + 4 + boundary.len()..idx + full_boundary_len) == Some(b"--\r\n")
        {
            return Ok((idx, full_boundary_len));
        }
    }

    Err(Error::Known("closing boundary not found"))
}

fn before_closing_boundary<'a, 'b>(input: &'a [u8], boundary: &'b [u8]) -> Res<'a, &'a [u8]> {
    let (before, len) = before_closing_boundary_idx(input, boundary)?;

    // JUSTIFICATION
    //  Benefit
    //      Improve performances by avoiding checking the range.
    //  Correctness
    //      The function returning indexes cannot return out of range.
    unsafe {
        Ok((
            input.get_unchecked(before + len..),
            input.get_unchecked(..before),
        ))
    }
}

fn before_closing_boundary_owned(
    mut input: Vec<u8>,
    boundary: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let (before, len) = before_closing_boundary_idx(&input, boundary)?;

    let before: Vec<u8> = input.drain(..before).collect();
    {
        let _boundary = input.drain(..len);
    }

    Ok((before, input))
}

pub fn parse_multipart<'a>(
    input: &'a [u8],
    parameters: HashMap<Cow<str>, Cow<str>>,
) -> Result<Vec<RawEntity<'a>>, Error> {
    let boundary = parameters
        .get("boundary")
        .ok_or(Error::Known("Missing boundary parameter"))?;
    let (input, mut parts) = many(&input, |i| before_boundary(i, boundary.as_bytes()))?;
    let (_epilogue, last_part) = before_closing_boundary(input, boundary.as_bytes())?;
    parts.push(last_part);
    parts.remove(0); // the prelude

    let mut entities = Vec::new();
    for part in parts {
        entities.push(super::entity::raw_entity(Cow::Borrowed(part))?);
    }

    Ok(entities)
}

pub fn parse_multipart_owned<'a>(
    mut input: Vec<u8>,
    parameters: HashMap<Cow<str>, Cow<str>>,
) -> Result<Vec<RawEntity<'a>>, Error> {
    let boundary = parameters
        .get("boundary")
        .ok_or(Error::Known("Missing boundary parameter"))?;
    let mut parts = Vec::new();
    while let Ok((before, len)) = before_boundary_idx(&input, boundary.as_bytes()) {
        let part: Vec<u8> = input.drain(..before).collect();
        let _boundary = input.drain(..len);
        parts.push(part);
    }

    let (_epilogue, last_part) = before_closing_boundary_owned(input, boundary.as_bytes())?;
    parts.push(last_part);
    parts.remove(0); // the prelude

    let mut entities = Vec::new();
    for part in parts {
        entities.push(super::entity::raw_entity(Cow::Owned(part))?);
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
            b"aeiouy",
            before_boundary(b"aeiouy\r\n--boundary\r\n", b"boundary")
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
