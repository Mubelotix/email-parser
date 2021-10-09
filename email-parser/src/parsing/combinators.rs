use crate::prelude::*;
use std::borrow::Cow;

#[inline]
pub(crate) fn newline<'a>(input: &'a [u8], error_message: &'static str) -> Res<'a, ()> {
    #[cfg(feature = "compatibility-fixes")]
    return tag2(input, b"\r\n", b"\n", error_message);

    #[cfg(not(feature = "compatibility-fixes"))]
    return tag(input, b"\r\n", error_message);
}

#[inline]
pub(crate) fn tag<'a>(
    input: &'a [u8],
    expected: &'static [u8],
    error_message: &'static str,
) -> Res<'a, ()> {
    debug_assert!(std::str::from_utf8(expected).is_ok());
    if input.starts_with(expected) {
        Ok((unsafe { input.get_unchecked(expected.len()..) }, ()))
    } else {
        Err(Error::Explicit(error_message))
    }
}

#[inline]
pub(crate) fn tag2<'a>(
    input: &'a [u8],
    expected1: &'static [u8],
    expected2: &'static [u8],
    error_message: &'static str,
) -> Res<'a, ()> {
    debug_assert!(std::str::from_utf8(expected1).is_ok());
    if input.starts_with(expected1) {
        Ok((unsafe { input.get_unchecked(expected1.len()..) }, ()))
    } else {
        debug_assert!(std::str::from_utf8(expected2).is_ok());
        if input.starts_with(expected2) {
            Ok((unsafe { input.get_unchecked(expected2.len()..) }, ()))
        } else {
            Err(Error::Explicit(error_message))
        }
    }
}

#[inline]
pub(crate) fn tag_no_case<'a>(
    input: &'a [u8],
    expected: &'static [u8],
    expected2: &'static [u8],
    error_message: &'static str,
) -> Res<'a, ()> {
    debug_assert_eq!(expected.len(), expected2.len());
    debug_assert!(std::str::from_utf8(expected).is_ok());
    debug_assert!(std::str::from_utf8(expected2).is_ok());

    #[cfg(debug_assertions)]
    for i in 0..expected.len() {
        if (expected[i].is_ascii_lowercase() && expected[i].to_ascii_uppercase() != expected2[i])
            || (expected[i].is_ascii_uppercase()
                && expected[i].to_ascii_lowercase() != expected2[i])
        {
            panic!("tag_no_case() is supposed to take opposite characters but it is not the case for {:?}", std::str::from_utf8(expected).unwrap());
        }
    }

    if input.len() < expected.len() {
        return Err(Error::Unknown(
            "Tag error, input is smaller than expected string",
        ));
    }

    for idx in 0..expected.len() {
        unsafe {
            if input.get_unchecked(idx) != expected.get_unchecked(idx)
                && input.get_unchecked(idx) != expected2.get_unchecked(idx)
            {
                return Err(Error::Explicit(error_message));
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
    Err(Error::Unknown("No match arm is matching the data"))
}

#[inline]
pub fn take_while<F>(input: &[u8], mut condition: F) -> Res<&str>
where
    F: FnMut(u8) -> bool,
{
    for i in 0..input.len() {
        unsafe {
            if !condition(*input.get_unchecked(i)) {
                return Ok((
                    input.get_unchecked(i..),
                    std::str::from_utf8_unchecked(input.get_unchecked(..i)),
                ));
            }
        }
    }
    Ok((&[], unsafe { std::str::from_utf8_unchecked(input) }))
}

#[inline]
pub fn take_while1<F>(input: &[u8], mut condition: F) -> Res<&str>
where
    F: FnMut(u8) -> bool,
{
    if let Some(character) = input.get(0) {
        if !condition(*character) {
            return Err(Error::Unknown("Expected at least one character matching"));
        }
    } else {
        return Err(Error::Unknown(
            "Expected at least one character matching, but there is no character",
        ));
    }

    for i in 1..input.len() {
        unsafe {
            if !condition(*input.get_unchecked(i)) {
                return Ok((
                    input.get_unchecked(i..),
                    std::str::from_utf8_unchecked(input.get_unchecked(..i)),
                ));
            }
        }
    }
    Ok((&[], unsafe { std::str::from_utf8_unchecked(input) }))
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
pub fn collect_many<'a, F>(mut input: &'a [u8], mut parser: F) -> Res<Cow<str>>
where
    F: FnMut(&'a [u8]) -> Res<Cow<str>>,
{
    let mut result = empty_string();

    while let Ok((new_input, new_result)) = parser(input) {
        input = new_input;
        add_string(&mut result, new_result);
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
pub fn triplet<'a, T, U, V, F, G, H>(
    input: &'a [u8],
    mut parser1: F,
    mut parser2: G,
    mut parser3: H,
) -> Res<(U, T, V)>
where
    F: FnMut(&'a [u8]) -> Res<U>,
    G: FnMut(&'a [u8]) -> Res<T>,
    H: FnMut(&'a [u8]) -> Res<V>,
{
    let (input, first) = parser1(input)?;
    let (input, second) = parser2(input)?;
    let (input, third) = parser3(input)?;

    Ok((input, (first, second, third)))
}

#[inline]
pub fn collect_pair<'a, F, G>(input: &'a [u8], mut parser1: F, mut parser2: G) -> Res<Cow<str>>
where
    F: FnMut(&'a [u8]) -> Res<Cow<str>>,
    G: FnMut(&'a [u8]) -> Res<Cow<str>>,
{
    let (input, mut first) = parser1(input)?;
    let (input, second) = parser2(input)?;
    add_string(&mut first, second);

    Ok((input, first))
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
        return Err(Error::Unknown("Expected a prefix"));
    }
    parser(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_add_test() {
        let data = b"abcdef";
        let mut data1 = from_slice(&data[..3]);
        let data2 = from_slice(&data[3..]);
        add_string(&mut data1, data2);

        let mut data3 = from_slice(&data[..2]);
        let data4 = from_slice(&data[3..]);
        add_string(&mut data3, data4);

        assert!(matches!(data1, std::borrow::Cow::Borrowed(_)));
        assert!(matches!(data3, std::borrow::Cow::Owned(_)));
    }

    #[test]
    fn test_optional() {
        assert!(
            optional(b"abcdef", |input| tag(input, b"efg", "TAG ERROR: Testing"))
                .1
                .is_none()
        );
        assert!(
            optional(b"abcdef", |input| tag(input, b"abc", "TAG ERROR: Testing"))
                .1
                .is_some()
        );
    }

    #[test]
    fn test_take_while() {
        assert_eq!(take_while(b"     abc", is_wsp).unwrap().1.len(), 5);
        assert_eq!(take_while(b"abc", is_wsp).unwrap().1.len(), 0);
    }

    #[test]
    fn test_tag() {
        assert!(tag(b"abc", b"def", "TAG ERROR: Testing").is_err());
        assert!(tag(b"abc", b"ab", "TAG ERROR: Testing").is_ok());
        assert_eq!(tag(b"abc", b"abc", "TAG ERROR: Testing").unwrap().0, b"");
        assert!(tag(b"abc", b"Ab", "TAG ERROR: Testing").is_err());
        assert!(tag_no_case(b"abc", b"Ab", b"aB", "TAG ERROR: Testing no case").is_ok());
    }
}
