use crate::prelude::*;

pub fn quoted_pair(input: &[u8]) -> Result<(&[u8], String), Error> {
    let (input, ()) = tag(input, b"\\")?;

    if let Some(character) = input.get(0) {
        if is_vchar(*character) || is_wsp(*character) {
            // index are already checked
            unsafe {
                Ok((
                    input.get_unchecked(1..),
                    String::Reference(input.get_unchecked(..1)),
                ))
            }
        } else {
            Err(Error::Known(
                "The quoted-pair character is no a vchar or a wsp.",
            ))
        }
    } else {
        Err(Error::Known("The quoted-pair has no second character."))
    }
}

pub fn quoted_string(input: &[u8]) -> Result<(&[u8], String), Error> {
    let input = if let Ok((input, _cfws)) = cfws(input) {
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

        let new_input = if let Ok((new_input, fws)) = fws(input) {
            additionnal_output += fws;
            new_input
        } else {
            input
        };

        let new_input = if let Ok((new_input, str)) = take_while1(new_input, is_qtext) {
            additionnal_output += str;
            new_input
        } else if let Ok((new_input, str)) = quoted_pair(new_input) {
            additionnal_output += str;
            new_input
        } else {
            break;
        };

        output += additionnal_output;
        input = new_input;
    }

    let input = if let Ok((input, fws)) = fws(input) {
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

    let input = if let Ok((input, _cfws)) = cfws(input) {
        input
    } else {
        input
    };

    Ok((input, output))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quoted_pair() {
        assert!(quoted_pair(b"\\rtest").is_ok());
        assert!(quoted_pair(b"\\ test").is_ok());

        assert_eq!(quoted_pair(b"\\rtest").unwrap().1, "r");
        assert_eq!(quoted_pair(b"\\ test").unwrap().1, " ");

        assert!(quoted_pair(b"\\").is_err());
        assert!(quoted_pair(b"\\\0").is_err());
        assert!(quoted_pair(b"test").is_err());
    }

    #[test]
    fn test_quoted_string() {
        assert_eq!(
            quoted_string(b" \"This\\ is\\ a\\ test\"").unwrap().1,
            "This is a test"
        );
        assert_eq!(
            quoted_string(b"\r\n  \"This\\ is\\ a\\ test\"  ")
                .unwrap()
                .1,
            "This is a test"
        );

        assert!(matches!(
            quoted_string(b"\r\n  \"This\\ is\\ a\\ test\"  ")
                .unwrap()
                .1,
            String::Owned(_)
        ));
        assert!(matches!(
            quoted_string(b"\r\n  \"hey\"  ").unwrap().1,
            String::Str(_)
        ));
    }
}
