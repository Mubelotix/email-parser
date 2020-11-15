use crate::mime::*;
use crate::prelude::*;
use std::borrow::Cow;

#[inline]
fn ignore_inline_cfws(input: &[u8]) -> Res<()> {
    triplet(
        input,
        |input| take_while(input, is_wsp),
        |input| Ok(optional(input, comment)),
        |input| take_while(input, is_wsp),
    )
    .map(|(i, _)| (i, ()))
}

#[inline]
fn token(input: &[u8]) -> Res<Cow<str>> {
    take_while1(input, |c| {
        c > 0x1F && c < 0x7F && !is_wsp(c) && !tspecial(c)
    })
}

pub fn mime_version(input: &[u8]) -> Res<(u8, u8)> {
    let (input, ()) = tag_no_case(input, b"MIME-Version:", b"mime-vERSION:")?;
    let (input, _) = optional(input, cfws);

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

    let (input, _cwfs) = ignore_inline_cfws(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (d1, d2)))
}

pub fn content_type(input: &[u8]) -> Res<(MimeType, Cow<str>, Vec<(Cow<str>, Cow<str>)>)> {
    let (input, ()) = tag_no_case(input, b"Content-Type:", b"cONTENT-tYPE:")?;
    let (input, _) = optional(input, cfws);

    let (input, mime_type) = match_parsers(
        input,
        &mut [
            |input| tag_no_case(input, b"text", b"TEXT").map(|(i, ())| (i, MimeType::Text)),
            |input| {
                tag_no_case(input, b"multipart", b"MULTIPART")
                    .map(|(i, ())| (i, MimeType::Multipart))
            },
            |input| {
                tag_no_case(input, b"application", b"APPLICATION")
                    .map(|(i, ())| (i, MimeType::Application))
            },
            |input| tag_no_case(input, b"image", b"IMAGE").map(|(i, ())| (i, MimeType::Image)),
            |input| tag_no_case(input, b"video", b"VIDEO").map(|(i, ())| (i, MimeType::Video)),
            |input| tag_no_case(input, b"audio", b"AUDIO").map(|(i, ())| (i, MimeType::Audio)),
            |input| {
                tag_no_case(input, b"message", b"MESSAGE").map(|(i, ())| (i, MimeType::Message))
            },
            |input| {
                // TODO ietf token
                let (input, mut name) = token(input)?;

                // convert to lowercase
                let mut change_needed = false;
                for c in name.chars() {
                    if c.is_uppercase() {
                        change_needed = true;
                    }
                }
                if change_needed {
                    name = Cow::Owned(name.to_ascii_lowercase());
                }

                Ok((input, MimeType::Other(name)))
            },
        ][..],
    )?;
    let (input, ()) = tag(input, b"/")?;
    let (input, sub_type) = token(input)?;

    fn parameter(input: &[u8]) -> Res<(Cow<str>, Cow<str>)> {
        let (input, _) = optional(input, cfws);
        let (input, ()) = tag(input, b";")?;
        let (input, _) = optional(input, cfws);
        let (input, mut attribute) = token(input)?;

        let mut change_needed = false;
        for c in attribute.chars() {
            if c.is_uppercase() {
                change_needed = true;
            }
        }
        if change_needed {
            attribute = Cow::Owned(attribute.to_ascii_lowercase());
        }

        let (input, ()) = tag(input, b"=")?;
        let (input, value) = match_parsers(input, &mut [token, quoted_string][..])?;

        Ok((input, (attribute, value)))
    }

    let (input, parameters) = many(input, parameter)?;

    let (input, ()) = ignore_inline_cfws(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (mime_type, sub_type, parameters)))
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

    #[test]
    fn test_content_type() {
        assert_eq!(content_type(b"Content-type: tExt/plain\r\n").unwrap().1.0, MimeType::Text);
        assert_eq!(content_type(b"Content-type: text/plain\r\n").unwrap().1.1, "plain");
        assert_eq!(content_type(b"Content-type: multIpart/unknown\r\n").unwrap().1.0, MimeType::Multipart);
        assert_eq!(content_type(b"Content-Type: text/plain; chaRSet=\"iso-8859-1\"\r\n").unwrap().1.2[0].0, "charset");
        assert_eq!(content_type(b"Content-Type: text/plain; charset=\"iso-8859-1\"\r\n").unwrap().1.2[0].1, "iso-8859-1");
        assert_eq!(content_type(b"Content-type: text/plain; charset=us-ascii (Plain text)\r\n").unwrap().1.2[0].1, "us-ascii");
        assert_eq!(content_type(b"Content-type: text/plain; charset=\"us-ascii\"\r\n").unwrap().1.2[0].1, "us-ascii");
        assert_eq!(content_type(b"Content-Type: multipart/alternative; \r\n\tboundary=\"_000_DB6P193MB0021E64E5870F10170A32CB8EB920DB6P193MB0021EURP_\"\r\n").unwrap().1.2[0].0, "boundary");
    }
}
