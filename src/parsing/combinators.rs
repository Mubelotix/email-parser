use crate::prelude::*;

#[inline]
pub(crate) fn tag<'a>(input: &'a [u8], expected: &'static [u8]) -> Res<'a, ()> {
    if input.starts_with(expected) {
        Ok((unsafe { input.get_unchecked(expected.len()..) }, ()))
    } else {
        Err(Error::Known("Tag error, data does not match"))
    }
}

#[inline]
pub(crate) fn tag_no_case<'a>(
    input: &'a [u8],
    expected: &'static [u8],
    expected2: &'static [u8],
) -> Res<'a, ()> {
    debug_assert_eq!(expected.len(), expected2.len());
    // TODO case check

    if input.len() < expected.len() {
        return Err(Error::Known(
            "Tag error, input is smaller than expected string",
        ));
    }

    for idx in 0..expected.len() {
        unsafe {
            if input.get_unchecked(idx) != expected.get_unchecked(idx)
                && input.get_unchecked(idx) != expected2.get_unchecked(idx)
            {
                return Err(Error::Known("Tag error, data does not match"));
            }
        }
    }

    Ok((unsafe { input.get_unchecked(expected.len()..) }, ()))
}

#[inline]
pub(crate) fn optional<'a, T, F>(input: &'a [u8], mut parser: F) -> (&'a [u8], Option<T>)
where
    F: FnMut(&'a [u8]) -> Res<T>,
{
    if let Ok((input, parser)) = parser(input) {
        (input, Some(parser))
    } else {
        (input, None)
    }
}

#[inline]
pub(crate) fn match_parsers<'a, T, F>(input: &'a [u8], parsers: &mut [F]) -> Res<'a, T>
where
    F: FnMut(&'a [u8]) -> Res<T>,
{
    for parser in parsers {
        let result = parser(input);
        if result.is_ok() {
            return result;
        }
    }
    Err(Error::Known("No match arm is matching the data"))
}

#[inline]
pub fn take_while<F>(input: &[u8], mut condition: F) -> Res<String>
where
    F: FnMut(u8) -> bool,
{
    for i in 0..input.len() {
        unsafe {
            if !condition(*input.get_unchecked(i)) {
                return Ok((
                    input.get_unchecked(i..),
                    String::Reference(input.get_unchecked(..i)),
                ));
            }
        }
    }
    Ok((&[], String::Reference(input)))
}

#[inline]
pub fn take_while1<F>(input: &[u8], mut condition: F) -> Res<String>
where
    F: FnMut(u8) -> bool,
{
    if let Some(character) = input.get(0) {
        if !condition(*character) {
            return Err(Error::Known("Expected at least one character matching"));
        }
    } else {
        return Err(Error::Known(
            "Expected at least one character matching, but there is no character",
        ));
    }

    for i in 1..input.len() {
        unsafe {
            if !condition(*input.get_unchecked(i)) {
                return Ok((
                    input.get_unchecked(i..),
                    String::Reference(input.get_unchecked(..i)),
                ));
            }
        }
    }
    Ok((&[], String::Reference(input)))
}

#[inline]
pub fn ignore_many<'a, T, F>(mut input: &'a [u8], mut parser: F) -> Res<()>
where
    F: FnMut(&'a [u8]) -> Res<T>,
{
    while let Ok((new_input, _result)) = parser(input) {
        input = new_input;
    }

    Ok((input, ()))
}

#[inline]
pub fn many<'a, T, F>(mut input: &'a [u8], mut parser: F) -> Res<Vec<T>>
where
    F: FnMut(&'a [u8]) -> Res<T>,
{
    let mut results = Vec::new();

    while let Ok((new_input, new_result)) = parser(input) {
        input = new_input;
        results.push(new_result);
    }

    Ok((input, results))
}

#[inline]
pub fn many1<'a, T, F>(input: &'a [u8], mut parser: F) -> Res<Vec<T>>
where
    F: FnMut(&'a [u8]) -> Res<T>,
{
    let mut results = Vec::new();
    let (mut input, first_result) = parser(input)?;
    results.push(first_result);

    while let Ok((new_input, new_result)) = parser(input) {
        input = new_input;
        results.push(new_result);
    }

    Ok((input, results))
}

#[inline]
pub fn collect_many<'a, F>(mut input: &'a [u8], mut parser: F) -> Res<String>
where
    F: FnMut(&'a [u8]) -> Res<String>,
{
    let mut result = String::new();

    while let Ok((new_input, new_result)) = parser(input) {
        input = new_input;
        result += new_result;
    }

    Ok((input, result))
}

#[inline]
pub fn pair<'a, T, U, F, G>(input: &'a [u8], mut parser1: F, mut parser2: G) -> Res<(U, T)>
where
    F: FnMut(&'a [u8]) -> Res<U>,
    G: FnMut(&'a [u8]) -> Res<T>,
{
    let (input, first) = parser1(input)?;
    let (input, second) = parser2(input)?;

    Ok((input, (first, second)))
}

#[inline]
pub fn collect_pair<'a, F, G>(input: &'a [u8], mut parser1: F, mut parser2: G) -> Res<String>
where
    F: FnMut(&'a [u8]) -> Res<String>,
    G: FnMut(&'a [u8]) -> Res<String>,
{
    let (input, first) = parser1(input)?;
    let (input, second) = parser2(input)?;

    Ok((input, first + second))
}

#[inline]
pub fn prefixed<'a, 'b, T, F>(
    mut input: &'a [u8],
    mut parser: F,
    prefix: &'b str,
) -> Result<(&'a [u8], T), Error>
where
    F: FnMut(&'a [u8]) -> Result<(&'a [u8], T), Error>,
{
    if input.starts_with(prefix.as_bytes()) {
        input = unsafe { input.get_unchecked(prefix.len()..) };
    } else {
        return Err(Error::Known("Expected a prefix"));
    }
    parser(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_add_test() {
        let data = b"abcdef";
        let data1 = String::Reference(&data[..3]);
        let data2 = String::Reference(&data[3..]);

        let data3 = String::Reference(&data[..2]);
        let data4 = String::Reference(&data[3..]);

        assert!(matches!(data1 + data2, String::Reference(_)));
        assert!(matches!(data3 + data4, String::Owned(_)));
    }

    #[test]
    fn test_optional() {
        assert!(optional(b"abcdef", |input| tag(input, b"efg")).1.is_none());
        assert!(optional(b"abcdef", |input| tag(input, b"abc")).1.is_some());
    }

    #[test]
    fn test_take_while() {
        assert_eq!(take_while(b"     abc", is_wsp).unwrap().1.len(), 5);
        assert_eq!(take_while(b"abc", is_wsp).unwrap().1.len(), 0);
    }

    #[test]
    fn test_tag() {
        assert!(tag(b"abc", b"def").is_err());
        assert!(tag(b"abc", b"ab").is_ok());
        assert_eq!(tag(b"abc", b"abc").unwrap().0, b"");
        assert!(tag(b"abc", b"Ab").is_err());
        assert!(tag_no_case(b"abc", b"Ab", b"aB").is_ok());
    }
}
