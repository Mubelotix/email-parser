use crate::prelude::*;

pub fn take_quoted_pair(input: &[u8]) -> Result<(&[u8], String), Error> {
    if input.starts_with(b"\\") {
        if let Some(character) = input.get(1) {
            if is_vchar(*character) || is_wsp(*character) {
                Ok((&input[2..], String::Reference(&input[1..2])))
            } else {
                Err(Error::Known(
                    "The quoted-pair character is no a vchar or a wsp.",
                ))
            }
        } else {
            Err(Error::Known("The quoted-pair has no second character."))
        }
    } else {
        Err(Error::Known("The quoted-pair does not start with a '\\'."))
    }
}

pub fn take_quoted_string(input: &[u8]) -> Result<(&[u8], String), Error> {
    let input = if let Ok((input, _cfws)) = take_cfws(input) {
        input
    } else {
        input
    };

    let mut input = if input.starts_with(b"\"") {
        &input[1..]
    } else {
        return Err(Error::Known("Quoted string must begin with a dquote"));
    };
    let mut output = String::Reference(&[]);

    loop {
        let mut additionnal_output = String::Reference(&[]);

        let new_input = if let Ok((new_input, fws)) = take_fws(input) {
            additionnal_output += fws;
            new_input
        } else {
            input
        };

        let new_input = if let Ok((new_input, str)) = take_while1(new_input, is_qtext) {
            additionnal_output += str;
            new_input
        } else if let Ok((new_input, str)) = take_quoted_pair(new_input) {
            additionnal_output += str;
            new_input
        } else {
            break;
        };

        output += additionnal_output;
        input = new_input;
    }

    let input = if let Ok((input, fws)) = take_fws(input) {
        output += fws;
        input
    } else {
        input
    };

    let input = if input.starts_with(b"\"") {
        &input[1..]
    } else {
        return Err(Error::Known("Quoted string must end with a dquote"));
    };

    let input = if let Ok((input, _cfws)) = take_cfws(input) {
        input
    } else {
        input
    };

    Ok((input, output))
}