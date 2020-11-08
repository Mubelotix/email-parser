use crate::prelude::*;

pub fn atom(mut input: &[u8]) -> Result<(&[u8], String), Error> {
    if let Ok((new_input, _)) = cfws(input) {
        input = new_input
    }
    let (mut input, atom) =
        take_while1(input, is_atext).map_err(|_| Error::Known("Atom required"))?;
    if let Ok((new_input, _)) = cfws(input) {
        input = new_input
    }
    Ok((input, atom))
}

pub fn dot_atom_text(input: &[u8]) -> Result<(&[u8], String), Error> {
    let (mut input, mut output) = take_while1(input, is_atext)?;

    loop {
        if input.starts_with(b".") {
            if let Ok((new_input, atom)) = take_while1(&input[1..], is_atext) {
                output += String::Reference(&input[..1]);
                input = new_input;
                output += atom;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok((input, output))
}

pub fn dot_atom(mut input: &[u8]) -> Result<(&[u8], String), Error> {
    if let Ok((new_input, _)) = cfws(input) {
        input = new_input
    }
    let (mut input, dot_atom) = dot_atom_text(input)?;
    if let Ok((new_input, _)) = cfws(input) {
        input = new_input
    }
    Ok((input, dot_atom))
}

pub fn word(input: &[u8]) -> Result<(&[u8], String), Error> {
    if let Ok((input, word)) = atom(input) {
        Ok((input, word))
    } else if let Ok((input, word)) = quoted_string(input) {
        Ok((input, word))
    } else {
        Err(Error::Known(
            "Word is not an atom and is not a quoted_string.",
        ))
    }
}

pub fn phrase(input: &[u8]) -> Result<(&[u8], Vec<String>), Error> {
    let mut words = Vec::new();
    let (mut input, first_word) = word(input)?;
    words.push(first_word);

    while let Ok((new_input, word)) = word(input) {
        input = new_input;
        words.push(word)
    }

    Ok((input, words))
}

pub fn unstructured(input: &[u8]) -> Result<(&[u8], String), Error> {
    let (mut input, output) = collect_many(input, |i| {
        collect_pair(
            i,
            |i| Ok(fws(i).unwrap_or((i, String::Reference(&[])))),
            |i| take_while1(i, is_vchar),
        )
    })?;

    while let Ok((new_input, _wsp)) = take_while1(input, is_wsp) {
        input = new_input;
    }

    Ok((input, output))
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
