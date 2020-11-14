use crate::prelude::*;

pub fn mime_version(input: &[u8]) -> Res<(u8, u8)> {
    let (input, ()) = tag_no_case(input, b"MIME-Version:", b"mime-vERSION:")?;
    let (input, _comment) = triplet(
        input,
        |input| take_while(input, is_wsp),
        |input| Ok(optional(input, comment)),
        |input| take_while(input, is_wsp),
    )?;

    fn u8_number(input: &[u8]) -> Res<u8> {
        let (mut input, mut number) = digit(input)?;

        while let Ok((new_input, new_digit)) = digit(input) {
            input = new_input;
            number = number
                .checked_mul(10)
                .ok_or(Error::Known("Overflow while reading u8."))?;
            number = number
                .checked_add(new_digit)
                .ok_or(Error::Known("Overflow while reading u8."))?;
        }

        Ok((input, number))
    }

    let (input, d1) = u8_number(input)?;
    let (input, ()) = tag(input, b".")?;
    let (input, d2) = u8_number(input)?;

    let (input, _cwfs) = triplet(
        input,
        |input| take_while(input, is_wsp),
        |input| Ok(optional(input, comment)),
        |input| take_while(input, is_wsp),
    )?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (d1, d2)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mime_version() {
        assert_eq!(mime_version(b"MIME-Version: 1.0\r\n").unwrap().1, (1, 0));
        assert_eq!(mime_version(b"MIME-VersIon: 1.2\r\n").unwrap().1, (1, 2));
        assert_eq!(
            mime_version(b"MIME-VersIon: (produced by MetaSend Vx.x) 2.0\r\n")
                .unwrap()
                .1,
            (2, 0)
        );
        assert_eq!(
            mime_version(b"MIME-VersIon: 214.25 (produced by MetaSend Vx.x)\r\n")
                .unwrap()
                .1,
            (214, 25)
        );
    }
}
