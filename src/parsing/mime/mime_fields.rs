use crate::mime::*;
use crate::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

use super::percent_encoding::decode_parameter;

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

fn parameter(input: &[u8]) -> Res<(Cow<str>, Option<u8>, bool, Cow<str>)> {
    let (input, _) = optional(input, cfws);
    let (input, ()) = tag(input, b";")?;
    let (input, _) = optional(input, cfws);
    let (input, (mut name, index, encoded)) = match_parsers(input, &mut[
        |input| {
            let (input, name) = take_while1(input, |c| {
                c > 0x1F && c < 0x7F && !is_wsp(c) && !tspecial(c) && c != b'*'
            })?;

            let (mut input, index) = optional(input, |input| pair(input, |input| tag(input, b"*"), |input| take_while1(input, is_digit)));
            let index = if let Some(((), index)) = index {
                Some(index.parse::<u8>().map_err(|_| Error::Known("Invalid index"))?)
            } else {
                None
            };

            let encoded = if input.get(0) == Some(&b'*') {
                input = &input[1..];
                true
            } else {
                false
            };

            if input.get(0) == Some(&b'=') {
                Ok((input, (name, index, encoded)))
            } else {
                Err(Error::Known("It wont work with this method"))
            }
        },
        |input| {
            let (input, name) = token(input)?;
            Ok((input, (name, None, false)))
        }
    ][..])?;

    let mut change_needed = false;
    for c in name.chars() {
        if c.is_uppercase() {
            change_needed = true;
        }
    }
    if change_needed {
        name = Cow::Owned(name.to_ascii_lowercase());
    }

    let (input, ()) = tag(input, b"=")?;
    let (input, value) = match_parsers(input, &mut [token, quoted_string][..])?;

    Ok((input, (name, index, encoded, value)))
}

pub fn content_type(input: &[u8]) -> Res<(MimeType, Cow<str>, HashMap<Cow<str>, Cow<str>>)> {
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

    let (input, parameters_vec) = many(input, parameter)?;
    let parameters = super::percent_encoding::collect_parameters(parameters_vec)?;

    let (input, ()) = ignore_inline_cfws(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (mime_type, sub_type, parameters)))
}

pub fn content_disposition(input: &[u8]) -> Res<Disposition> {
    use crate::parsing::time::date_time;

    let (input, ()) = tag_no_case(input, b"Content-Disposition:", b"cONTENT-dISPOSITION:")?;
    let (input, _) = optional(input, cfws);

    let (mut input, disposition_type) = match_parsers(
        input,
        &mut [
            |input| {
                tag_no_case(input, b"inline", b"INLINE").map(|(i, ())| (i, DispositionType::Inline))
            },
            |input| {
                tag_no_case(input, b"attachment", b"ATTACHMENT")
                    .map(|(i, ())| (i, DispositionType::Attachment))
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

                Ok((input, DispositionType::Unknown(name)))
            },
        ][..],
    )?;

    let mut disposition = Disposition {
        disposition_type,
        unstructured: HashMap::new(),
        creation_date: None,
        modification_date: None,
        read_date: None,
        filename: None,
    };
    let mut parameters_vec = Vec::new();
    loop {
        fn filename_parameter(input: &[u8]) -> Res<Cow<str>> {
            let (input, _) = optional(input, cfws);
            let (input, ()) = tag(input, b";")?;
            let (input, _) = optional(input, cfws);
            let (input, ()) = tag_no_case(input, b"filename", b"FILENAME")?;

            let (input, ()) = tag(input, b"=")?;
            let (input, value) = match_parsers(input, &mut [token, quoted_string][..])?;

            Ok((input, value))
        }

        fn date_parameter<'a>(
            input: &'a [u8],
            name: &'static [u8],
            name_uppercase: &'static [u8],
        ) -> Res<'a, DateTime> {
            let (input, _) = optional(input, cfws);
            let (input, ()) = tag(input, b";")?;
            let (input, _) = optional(input, cfws);
            let (input, ()) = tag_no_case(input, name, name_uppercase)?;

            let (input, ()) = tag(input, b"=\"")?;
            let (input, value) = date_time(input)?;
            let (input, ()) = tag(input, b"\"")?;

            Ok((input, value))
        }

        if let Ok((new_input, value)) = filename_parameter(input) {
            disposition.filename = Some(value);
            input = new_input;
        } else if let Ok((new_input, value)) =
            date_parameter(input, b"creation-date", b"CREATION-DATE")
        {
            disposition.creation_date = Some(value);
            input = new_input;
        } else if let Ok((new_input, value)) =
            date_parameter(input, b"modification-date", b"MODIFICATION-DATE")
        {
            disposition.modification_date = Some(value);
            input = new_input;
        } else if let Ok((new_input, value)) = date_parameter(input, b"read-date", b"READ-DATE") {
            disposition.read_date = Some(value);
            input = new_input;
        } else if let Ok((new_input, (name, index, encoded, value))) = parameter(input) {
            parameters_vec.push((name, value));
            input = new_input;
        } else {
            break;
        }
    }
    disposition.unstructured = super::percent_encoding::collect_parameters(parameters_vec)?;

    let (input, ()) = ignore_inline_cfws(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, disposition))
}

pub fn content_transfer_encoding(input: &[u8]) -> Res<ContentTransferEncoding> {
    let (input, ()) = tag_no_case(
        input,
        b"Content-Transfer-Encoding:",
        b"cONTENT-tRANSFER-eNCODING:",
    )?;
    let (input, _) = optional(input, cfws);

    let (input, encoding) = match_parsers(
        input,
        &mut [
            |input| {
                tag_no_case(input, b"7bit", b"7BIT")
                    .map(|(i, ())| (i, ContentTransferEncoding::SevenBit))
            },
            |input| {
                tag_no_case(input, b"quoted-printable", b"QUOTED-PRINTABLE")
                    .map(|(i, ())| (i, ContentTransferEncoding::QuotedPrintable))
            },
            |input| {
                tag_no_case(input, b"base64", b"BASE64")
                    .map(|(i, ())| (i, ContentTransferEncoding::Base64))
            },
            |input| {
                tag_no_case(input, b"8bit", b"8BIT")
                    .map(|(i, ())| (i, ContentTransferEncoding::HeightBit))
            },
            |input| {
                tag_no_case(input, b"binary", b"BINARY")
                    .map(|(i, ())| (i, ContentTransferEncoding::Binary))
            },
            |input| {
                let (input, mut encoding) = token(input)?;

                // convert to lowercase
                let mut change_needed = false;
                for c in encoding.chars() {
                    if c.is_uppercase() {
                        change_needed = true;
                    }
                }
                if change_needed {
                    encoding = Cow::Owned(encoding.to_ascii_lowercase());
                }

                Ok((input, ContentTransferEncoding::Other(encoding)))
            },
        ][..],
    )?;

    let (input, _cwfs) = ignore_inline_cfws(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, encoding))
}

pub fn content_id(input: &[u8]) -> Res<(Cow<str>, Cow<str>)> {
    let (input, ()) = tag_no_case(input, b"Content-ID:", b"cONTENT-id:")?;
    let (input, id) = crate::parsing::address::message_id(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, id))
}

pub fn content_description(input: &[u8]) -> Res<Cow<str>> {
    let (input, ()) = tag_no_case(input, b"Content-Description:", b"cONTENT-dESCRIPTION:")?;
    let (input, description) = mime_unstructured(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, description))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_content_disposition() {
        println!(
            "{:?}",
            content_disposition(b"Content-Disposition: inline\r\n")
                .unwrap()
                .1
        );
        println!("{:?}", content_disposition(b"Content-Disposition: attachment; filename=genome.jpeg;\r\n modification-date=\"Wed, 12 Feb 1997 16:29:51 -0500\"\r\n").unwrap().1);
        println!(
            "{:?}",
            content_disposition(b"Content-Disposition: attachment\r\n")
                .unwrap()
                .1
        );
    }

    #[test]
    fn test_content_id() {
        assert_eq!(
            content_id(b"Content-ID: <123456@mubelotix.dev>\r\n")
                .unwrap()
                .1
                 .0,
            "123456"
        );
        assert_eq!(
            content_id(b"cOntent-id: <qpfpsqfh@gmail.com>\r\n")
                .unwrap()
                .1
                 .1,
            "gmail.com"
        );
    }

    #[test]
    fn test_content_description() {
        assert_eq!(
            content_description(
                b"Content-Description:a picture of the Space Shuttle Endeavor.\r\n"
            )
            .unwrap()
            .1,
            "a picture of the Space Shuttle Endeavor."
        );
        assert_eq!(
            content_description(b"Content-DeScription:Ferris the crab\r\n")
                .unwrap()
                .1,
            "Ferris the crab"
        );
    }

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
        assert_eq!(
            content_type(b"Content-type: tExt/plain\r\n").unwrap().1 .0,
            MimeType::Text
        );
        println!(
            "{:?}", content_type(b"Content-Type: message/external-body; access-type=URL;\r\n URL*0=\"ftp://\";\r\n URL*1=\"cs.utk.edu/pub/moore/bulk-mailer/bulk-mailer.tar\"\r\n").unwrap().1,
        );
        println!(
            "{:?}", content_type(b"Content-Type: application/x-stuff;\r\n title*0*=us-ascii'en'This%20is%20even%20more%20;\r\n title*1*=%2A%2A%2Afun%2A%2A%2A%20;\r\n title*2=\"isn't it!\"\r\n").unwrap().1,
        );
        println!(
            "{:?}", parameter(b";\r\n URL*0=\"ftp://\"").unwrap().1,
        );
        assert_eq!(
            content_type(b"Content-type: text/plain\r\n").unwrap().1 .1,
            "plain"
        );
        assert_eq!(
            content_type(b"Content-type: multIpart/unknown\r\n")
                .unwrap()
                .1
                 .0,
            MimeType::Multipart
        );
        assert_eq!(
            content_type(b"Content-Type: text/plain; chaRSet=\"iso-8859-1\"\r\n")
                .unwrap()
                .1
                 .2
                .get("charset")
                .unwrap(),
            "iso-8859-1"
        );
        assert_eq!(
            content_type(b"Content-Type: text/plain; charset=\"iso-8859-1\"\r\n")
                .unwrap()
                .1
                 .2
                .get("charset")
                .unwrap(),
            "iso-8859-1"
        );
        assert_eq!(
            content_type(b"Content-type: text/plain; charset=us-ascii (Plain text)\r\n")
                .unwrap()
                .1
                 .2
                .get("charset")
                .unwrap(),
            "us-ascii"
        );
        assert_eq!(
            content_type(b"Content-type: text/plain; charset=\"us-ascii\"\r\n")
                .unwrap()
                .1
                 .2
                .get("charset")
                .unwrap(),
            "us-ascii"
        );
        assert_eq!(content_type(b"Content-Type: multipart/alternative; \r\n\tboundary=\"_000_DB6P193MB0021E64E5870F10170A32CB8EB920DB6P193MB0021EURP_\"\r\n").unwrap().1.2.get("boundary").unwrap(), "_000_DB6P193MB0021E64E5870F10170A32CB8EB920DB6P193MB0021EURP_");
    }

    #[test]
    fn test_content_transfer_encoding() {
        assert_eq!(
            content_transfer_encoding(b"Content-Transfer-Encoding: 7bit\r\n")
                .unwrap()
                .1,
            ContentTransferEncoding::SevenBit
        );
        assert_eq!(
            content_transfer_encoding(b"Content-Transfer-Encoding: binary (invalid) \r\n")
                .unwrap()
                .1,
            ContentTransferEncoding::Binary
        );
        assert_eq!(
            content_transfer_encoding(b"Content-Transfer-Encoding: (not readable) base64 \r\n")
                .unwrap()
                .1,
            ContentTransferEncoding::Base64
        );
    }
}
