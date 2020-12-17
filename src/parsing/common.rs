use crate::prelude::*;
use std::borrow::Cow;

pub fn lowercase(mut value: Cow<str>) -> Cow<str> {
    let mut change_needed = false;
    for c in value.chars() {
        if c.is_uppercase() {
            change_needed = true;
        }
    }
    if change_needed {
        value = Cow::Owned(value.to_ascii_lowercase());
    }
    value
}

pub fn atom(mut input: &[u8]) -> Res<&str> {
    if let Ok((new_input, _)) = cfws(input) {
        input = new_input
    }
    let (mut input, atom) =
        take_while1(input, is_atext).map_err(|_| Error::Unknown ("Atom required"))?;
    if let Ok((new_input, _)) = cfws(input) {
        input = new_input
    }
    Ok((input, atom))
}

pub fn dot_atom_text(input: &[u8]) -> Res<Cow<str>> {
    let (mut input, output) = take_while1(input, is_atext)?;
    let mut output = Cow::Borrowed(output);

    loop {
        if input.starts_with(b".") {
            if let Ok((new_input, atom)) = if cfg!(feature = "compatibility-fixes") {
                take_while(&input[1..], is_atext)
            } else {
                take_while1(&input[1..], is_atext)
            } {
                add_string(&mut output, from_slice(&input[..1]));
                input = new_input;
                add_string(&mut output, Cow::Borrowed(atom));
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok((input, output))
}

pub fn dot_atom(mut input: &[u8]) -> Result<(&[u8], Cow<str>), Error> {
    if let Ok((new_input, _)) = cfws(input) {
        input = new_input
    }
    let (mut input, dot_atom) = dot_atom_text(input)?;
    if let Ok((new_input, _)) = cfws(input) {
        input = new_input
    }
    Ok((input, dot_atom))
}

pub fn word(input: &[u8]) -> Res<Cow<str>> {
    match_parsers(
        input,
        &mut [
            |input| {
                let (input, value) = atom(input)?;
                Ok((input, Cow::Borrowed(value)))
            },
            |input| quoted_string(input),
        ][..],
    )
}

pub fn phrase(input: &[u8]) -> Result<(&[u8], Vec<Cow<str>>), Error> {
    #[cfg(feature = "mime")]
    fn word(input: &[u8]) -> Res<Cow<str>> {
        match_parsers(
            input,
            &mut [
                (|i| {
                    let (i, _) = optional(i, fws);
                    crate::parsing::mime::encoded_headers::encoded_word(i)
                }),
                (|i| crate::parsing::common::word(i)),
            ][..],
        )
    }

    let mut words = Vec::new();
    let (mut input, first_word) = word(input)?;
    words.push(first_word);

    while let Ok((new_input, word)) = word(input) {
        input = new_input;
        words.push(word)
    }

    Ok((input, words))
}

pub fn unstructured(input: &[u8]) -> Result<(&[u8], Cow<str>), Error> {
    let (mut input, output) = collect_many(input, |i| {
        collect_pair(
            i,
            |i| Ok(fws(i).unwrap_or((i, empty_string()))),
            |i| {
                let (input, value) = take_while1(i, is_vchar)?;
                Ok((input, Cow::Borrowed(value)))
            },
        )
    })?;

    while let Ok((new_input, _wsp)) = take_while1(input, is_wsp) {
        input = new_input;
    }

    Ok((input, output))
}

#[cfg(feature = "mime")]
pub fn mime_unstructured(input: &[u8]) -> Res<Cow<str>> {
    let mut previous_was_encoded = false;
    let (mut input, output) = collect_many(input, |i| {
        let (i, mut wsp) = fws(i).unwrap_or((i, empty_string()));

        if let Ok((i, text)) = crate::parsing::mime::encoded_headers::encoded_word(i) {
            if previous_was_encoded {
                return Ok((i, text));
            } else {
                previous_was_encoded = true;
                add_string(&mut wsp, text);
                return Ok((i, wsp));
            }
        } else if let Ok((i, text)) = take_while1(i, is_vchar) {
            previous_was_encoded = false;
            add_string(&mut wsp, Cow::Borrowed(text));
            return Ok((i, wsp));
        }
        Err(Error::Unknown ("No match arm is matching the data"))
    })?;

    while let Ok((new_input, _wsp)) = take_while1(input, is_wsp) {
        input = new_input;
    }

    Ok((input, output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "mime")]
    #[test]
    fn test_encoded_unstructured() {
        assert_eq!(
            "the quick brown fox jumps over Chloé Helloco",
            mime_unstructured(
                b"the quick brown fox jumps\r\n over =?UTF-8?Q?Chlo=C3=A9_Helloco?=   "
            )
            .unwrap()
            .1
        );

        assert_eq!("a", mime_unstructured(b"=?ISO-8859-1?Q?a?=").unwrap().1);
        assert_eq!("a b", mime_unstructured(b"=?ISO-8859-1?Q?a?= b").unwrap().1);
        assert_eq!(
            "ab",
            mime_unstructured(b"=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?=")
                .unwrap()
                .1
        );
        assert_eq!(
            "ab",
            mime_unstructured(b"=?ISO-8859-1?Q?a?=  =?ISO-8859-1?Q?b?=")
                .unwrap()
                .1
        );
        assert_eq!(
            "ab",
            mime_unstructured(b"=?ISO-8859-1?Q?a?=\r\n  =?ISO-8859-1?Q?b?=")
                .unwrap()
                .1
        );
        assert_eq!("a b", mime_unstructured(b"=?ISO-8859-1?Q?a_b?=").unwrap().1);
        assert_eq!(
            "a b",
            mime_unstructured(b"=?ISO-8859-1?Q?a?= =?ISO-8859-2?Q?_b?=")
                .unwrap()
                .1
        );
    }

    #[cfg(feature = "mime")]
    #[test]
    fn test_encoded_phrase() {
        assert_eq!(
            phrase(b"Lou =?UTF-8?Q?Dorl=C3=A9ans?=").unwrap().1,
            vec!["Lou", "Dorléans"]
        );

        assert_eq!(
            phrase(b"=?ISO-8859-1?Q?Andr=E9?= Pirard").unwrap().1,
            vec!["André", "Pirard"]
        );

        assert_eq!(
            phrase(b" =?US-ASCII?Q?Keith_Moore?=").unwrap().1,
            vec!["Keith Moore"]
        );
    }

    #[test]
    fn test_word_and_phrase() {
        assert_eq!(word(b" this is a \"rust\\ test\" ").unwrap().1, "this");
        assert_eq!(
            phrase(b" this is a \"rust\\ test\" ").unwrap().1,
            vec!["this", "is", "a", "rust test"]
        );
    }

    #[test]
    fn test_unstructured() {
        assert_eq!(
            unstructured(b"the quick brown fox jumps\r\n over the lazy dog   ")
                .unwrap()
                .1,
            "the quick brown fox jumps over the lazy dog"
        );
    }

    #[test]
    fn test_atom() {
        assert_eq!(atom(b"this is a test").unwrap().1, "this");
        assert_eq!(atom(b"   averylongatom ").unwrap().1, "averylongatom");
        assert_eq!(
            dot_atom_text(b"this.is.a.test").unwrap().1,
            "this.is.a.test"
        );
        assert_eq!(dot_atom(b"  this.is.a.test ").unwrap().1, "this.is.a.test");
    }
}
