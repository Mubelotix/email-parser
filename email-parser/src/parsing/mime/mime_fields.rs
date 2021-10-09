use crate::mime::*;
use crate::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

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
fn token(input: &[u8]) -> Res<&str> {
    take_while1(input, |c| {
        c > 0x1F && c < 0x7F && !is_wsp(c) && !tspecial(c)
    })
}

pub fn mime_version(input: &[u8]) -> Res<(u8, u8)> {
    let (input, ()) = tag_no_case(
        input,
        b"MIME-Version:",
        b"mime-vERSION:",
        "TAG NO CASE ERROR: Header name (Mime-Version) does not match.",
    )?;
    let (input, _) = optional(input, cfws);

    fn u8_number(input: &[u8]) -> Res<u8> {
        let (mut input, mut number) = digit(input)?;

        while let Ok((new_input, new_digit)) = digit(input) {
            input = new_input;
            number = number
                .checked_mul(10)
                .ok_or(Error::Unknown("Overflow while reading u8."))?;
            number = number
                .checked_add(new_digit)
                .ok_or(Error::Unknown("Overflow while reading u8."))?;
        }

        Ok((input, number))
    }

    let (input, d1) = u8_number(input)?;
    let (input, ()) = tag(
        input,
        b".",
        "TAG ERROR: A MIME version's major version number must be followed by a `.`.",
    )?;
    let (input, d2) = u8_number(input)?;

    let (input, _cwfs) = ignore_inline_cfws(input)?;
    let (input, ()) = newline(
        input,
        "TAG ERROR: A header (`MIME-Version` in this case) must end with a CRLF sequence.",
    )?;

    Ok((input, (d1, d2)))
}

fn parameter(input: &[u8]) -> Res<(Cow<str>, Option<u8>, bool, Cow<str>)> {
    let (input, _) = optional(input, cfws);
    let (input, ()) = tag(
        input,
        b";",
        "TAG ERROR: A MIME parameter must start with `;`.",
    )?;
    let (input, _) = optional(input, cfws);
    let (input, (mut name, index, encoded)) = match_parsers(
        input,
        &mut [
            |input| {
                let (input, name) = take_while1(input, |c| {
                    c > 0x1F && c < 0x7F && !is_wsp(c) && !tspecial(c) && c != b'*'
                })?;

                let (mut input, index) = optional(input, |input| {
                    pair(
                        input,
                        |input| {
                            tag(
                                input,
                                b"*",
                                "TAG ERROR: An indexed MIME parameter name must contain a `*`.",
                            )
                        },
                        |input| take_while1(input, is_digit),
                    )
                });
                let index = if let Some(((), index)) = index {
                    Some(
                        index
                            .parse::<u8>()
                            .map_err(|_| Error::Unknown("Invalid index"))?,
                    )
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
                    Ok((input, (Cow::Borrowed(name), index, encoded)))
                } else {
                    Err(Error::Unknown("It wont work with this method"))
                }
            },
            |input| {
                let (input, name) = token(input)?;
                Ok((input, (Cow::Borrowed(name), None, false)))
            },
        ][..],
    )?;

    name = lowercase(name);

    let (input, ()) = tag(
        input,
        b"=",
        "TAG ERROR: A MIME parameter name must be followed by a `=`.",
    )?;
    let (input, value) = match_parsers(
        input,
        &mut [
            |input| {
                let (input, value) = token(input)?;
                Ok((input, Cow::Borrowed(value)))
            },
            |input| quoted_string(input),
        ][..],
    )?;

    Ok((input, (name, index, encoded, value)))
}

pub fn content_type(input: &[u8]) -> Res<(ContentType, Cow<str>, HashMap<Cow<str>, Cow<str>>)> {
    let (input, ()) = tag_no_case(
        input,
        b"Content-Type:",
        b"cONTENT-tYPE:",
        "TAG NO CASE ERROR: Header name (Content-Type) does not match.",
    )?;
    let (input, _) = optional(input, cfws);

    let (input, mime_type) = match_parsers(
        input,
        &mut [
            |input| {
                tag_no_case(
                    input,
                    b"text",
                    b"TEXT",
                    "TAG NO CASE ERROR: In a content type header, `text` type does not match.",
                )
                .map(|(i, ())| (i, ContentType::Text))
            },
            |input| {
                tag_no_case(
                    input,
                    b"multipart",
                    b"MULTIPART",
                    "TAG NO CASE ERROR: In a content type header, `multipart` type does not match.",
                )
                .map(|(i, ())| (i, ContentType::Multipart))
            },
            |input| {
                tag_no_case(input, b"application", b"APPLICATION", "TAG NO CASE ERROR: In a content type header, `application` type does not match.")
                    .map(|(i, ())| (i, ContentType::Application))
            },
            |input| {
                tag_no_case(
                    input,
                    b"image",
                    b"IMAGE",
                    "TAG NO CASE ERROR: In a content type header, `image` type does not match.",
                )
                .map(|(i, ())| (i, ContentType::Image))
            },
            |input| {
                tag_no_case(
                    input,
                    b"video",
                    b"VIDEO",
                    "TAG NO CASE ERROR: In a content type header, `video` type does not match.",
                )
                .map(|(i, ())| (i, ContentType::Video))
            },
            |input| {
                tag_no_case(
                    input,
                    b"audio",
                    b"AUDIO",
                    "TAG NO CASE ERROR: In a content type header, `audio` type does not match.",
                )
                .map(|(i, ())| (i, ContentType::Audio))
            },
            |input| {
                tag_no_case(
                    input,
                    b"message",
                    b"MESSAGE",
                    "TAG NO CASE ERROR: In a content type header, `message` type does not match.",
                )
                .map(|(i, ())| (i, ContentType::Message))
            },
            |input| {
                // TODO ietf token
                let (input, name) = token(input)?;
                let name = lowercase(Cow::Borrowed(name));

                Ok((input, ContentType::Unknown(name)))
            },
        ][..],
    )?;
    let (input, ()) = tag(
        input,
        b"/",
        "TAG ERROR: A MIME content type must have a `/` separating the type and the subtype.",
    )?;
    let (input, subtype) = token(input)?;
    let subtype = lowercase(Cow::Borrowed(subtype));

    let (input, parameters_vec) = many(input, parameter)?;
    let parameters = super::percent_encoding::collect_parameters(parameters_vec)?;

    let (input, ()) = ignore_inline_cfws(input)?;
    let (input, ()) = newline(
        input,
        "TAG ERROR: A header (`Content-Type` in this case) must end with a CRLF sequence.",
    )?;

    Ok((input, (mime_type, subtype, parameters)))
}

pub fn content_disposition(input: &[u8]) -> Res<Disposition> {
    use crate::parsing::time::date_time;

    let (input, ()) = tag_no_case(
        input,
        b"Content-Disposition:",
        b"cONTENT-dISPOSITION:",
        "TAG NO CASE ERROR: Header name (Content-Disposition) does not match.",
    )?;
    let (input, _) = optional(input, cfws);

    let (mut input, disposition_type) = match_parsers(
        input,
        &mut [
            |input| {
                tag_no_case(input, b"inline", b"INLINE", "TAG NO CASE ERROR: In a content disposition header, `inline` disposition does not match.").map(|(i, ())| (i, DispositionType::Inline))
            },
            |input| {
                tag_no_case(input, b"attachment", b"ATTACHMENT", "TAG NO CASE ERROR: In a content disposition header, `attachment` disposition does not match.")
                    .map(|(i, ())| (i, DispositionType::Attachment))
            },
            |input| {
                // TODO ietf token
                let (input, name) = token(input)?;
                let name = lowercase(Cow::Borrowed(name));

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
            let (input, ()) = tag(input, b";", "TAG ERROR: In a Content-Disposition header, a filename parameter must start with a `;`.")?;
            let (input, _) = optional(input, cfws);
            let (input, ()) = tag_no_case(input, b"filename", b"FILENAME", "TAG NO CASE ERROR: In a Content-Disposition header, the name of the parameter does not match a filename parameter.")?;

            let (input, ()) = tag(input, b"=", "TAG ERROR: In a Content-Disposition header, a filename parameter value must be preceded by a `=`.")?;
            let (input, value) = match_parsers(
                input,
                &mut [
                    |input| {
                        let (input, value) = token(input)?;
                        Ok((input, Cow::Borrowed(value)))
                    },
                    |input| quoted_string(input),
                ][..],
            )?;

            Ok((input, value))
        }

        fn date_parameter<'a>(
            input: &'a [u8],
            name: &'static [u8],
            name_uppercase: &'static [u8],
        ) -> Res<'a, DateTime> {
            let (input, _) = optional(input, cfws);
            let (input, ()) = tag(input, b";", "TAG ERROR: In a Content-Disposition header, a date parameter must start with a `;`.")?;
            let (input, _) = optional(input, cfws);
            let (input, ()) = tag_no_case(input, name, name_uppercase, "TAG NO CASE ERROR: In a Content-Disposition header, the name of the parameter does not match a date parameter.")?;

            let (input, ()) = tag(input, b"=\"", "TAG ERROR: In a Content-Disposition header, a date parameter value must be preceded by `=\"`.")?;
            let (input, value) = date_time(input)?;
            let (input, ()) = tag(input, b"\"", "TAG ERROR: In a Content-Disposition header, a date parameter value must be closed by a `\"`.")?;

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
            parameters_vec.push((name, index, encoded, value));
            input = new_input;
        } else {
            break;
        }
    }
    disposition.unstructured = super::percent_encoding::collect_parameters(parameters_vec)?;

    let (input, ()) = ignore_inline_cfws(input)?;
    let (input, ()) = newline(
        input,
        "TAG ERROR: A header (`Content-Disposition` in this case) must end with a CRLF sequence.",
    )?;

    Ok((input, disposition))
}

pub fn content_transfer_encoding(input: &[u8]) -> Res<ContentTransferEncoding> {
    let (input, ()) = tag_no_case(
        input,
        b"Content-Transfer-Encoding:",
        b"cONTENT-tRANSFER-eNCODING:",
        "TAG NO CASE ERROR: Header name (Content-Transfer-Encoding) does not match.",
    )?;
    let (input, _) = optional(input, cfws);

    let (input, encoding) = match_parsers(
        input,
        &mut [
            |input| {
                tag_no_case(input, b"7bit", b"7BIT", "TAG NO CASE ERROR: In a content transfer encoding header, `7bit` encoding does not match.")
                    .map(|(i, ())| (i, ContentTransferEncoding::SevenBit))
            },
            |input| {
                tag_no_case(input, b"quoted-printable", b"QUOTED-PRINTABLE", "TAG NO CASE ERROR: In a content transfer encoding header, `quoted-printable` encoding does not match.")
                    .map(|(i, ())| (i, ContentTransferEncoding::QuotedPrintable))
            },
            |input| {
                tag_no_case(input, b"base64", b"BASE64", "TAG NO CASE ERROR: In a content transfer encoding header, `base64` encoding does not match.")
                    .map(|(i, ())| (i, ContentTransferEncoding::Base64))
            },
            |input| {
                tag_no_case(input, b"8bit", b"8BIT", "TAG NO CASE ERROR: In a content transfer encoding header, `8bit` encoding does not match.")
                    .map(|(i, ())| (i, ContentTransferEncoding::HeightBit))
            },
            |input| {
                tag_no_case(input, b"binary", b"BINARY", "TAG NO CASE ERROR: In a content transfer encoding header, `binary` encoding does not match.")
                    .map(|(i, ())| (i, ContentTransferEncoding::Binary))
            },
            |input| {
                let (input, encoding) = token(input)?;
                let encoding = lowercase(Cow::Borrowed(encoding));

                Ok((input, ContentTransferEncoding::Unknown(encoding)))
            },
        ][..],
    )?;

    let (input, _cwfs) = ignore_inline_cfws(input)?;
    let (input, ()) = newline(input, "TAG ERROR: A header (`Content-Transfer-Encoding` in this case) must end with a CRLF sequence.")?;

    Ok((input, encoding))
}

pub fn content_id(input: &[u8]) -> Res<(Cow<str>, Cow<str>)> {
    let (input, ()) = tag_no_case(
        input,
        b"Content-ID:",
        b"cONTENT-id:",
        "TAG NO CASE ERROR: Header name (Content-ID) does not match.",
    )?;
    let (input, id) = crate::parsing::address::message_id(input)?;
    let (input, ()) = newline(
        input,
        "TAG ERROR: A header (`Content-ID` in this case) must end with a CRLF sequence.",
    )?;

    Ok((input, id))
}

pub fn content_description(input: &[u8]) -> Res<Cow<str>> {
    let (input, ()) = tag_no_case(
        input,
        b"Content-Description:",
        b"cONTENT-dESCRIPTION:",
        "TAG NO CASE ERROR: Header name (Content-Description) does not match.",
    )?;
    let (input, description) = mime_unstructured(input)?;
    let (input, ()) = newline(
        input,
        "TAG ERROR: A header (`Content-Description` in this case) must end with a CRLF sequence.",
    )?;

    Ok((input, description))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_content_disposition() {
        assert_eq!(
            Disposition {
                disposition_type: DispositionType::Inline,
                filename: None,
                creation_date: None,
                modification_date: None,
                read_date: None,
                unstructured: HashMap::new()
            },
            content_disposition(b"Content-Disposition: inline\r\n")
                .unwrap()
                .1
        );
        assert_eq!(Disposition { disposition_type: DispositionType::Attachment, filename: Some("genome.jpeg".into()), creation_date: None, modification_date: Some(DateTime { day_name: Some(Day::Wednesday), date: Date { day: 12, month: Month::February, year: 1997 }, time: TimeWithZone { time: Time { hour: 16, minute: 29, second: 51 }, zone: Zone { sign: false, hour_offset: 5, minute_offset: 0 } } }), read_date: None, unstructured: HashMap::new() }, content_disposition(b"Content-Disposition: attachment; filename=genome.jpeg;\r\n modification-date=\"Wed, 12 Feb 1997 16:29:51 -0500\"\r\n").unwrap().1);
        assert_eq!(
            Disposition {
                disposition_type: DispositionType::Attachment,
                filename: None,
                creation_date: None,
                modification_date: None,
                read_date: None,
                unstructured: vec![("param".into(), "foobar".into())]
                    .into_iter()
                    .collect()
            },
            content_disposition(
                b"Content-Disposition: attachment; param*0=foo;\r\n param*1=bar\r\n"
            )
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
            ContentType::Text
        );
        assert_eq!(
            (ContentType::Message, "external-body".into(), vec![("access-type".into(), "URL".into()), ("url".into(), "ftp://cs.utk.edu/pub/moore/bulk-mailer/bulk-mailer.tar".into())].into_iter().collect()), content_type(b"Content-Type: message/external-body; access-type=URL;\r\n URL*0=\"ftp://\";\r\n URL*1=\"cs.utk.edu/pub/moore/bulk-mailer/bulk-mailer.tar\"\r\n").unwrap().1,
        );
        assert_eq!(
            (ContentType::Application, "x-stuff".into(), vec![("title".into(), "This is even more ***fun*** isn\'t it!".into())].into_iter().collect()), content_type(b"Content-Type: application/x-stuff;\r\n title*0*=us-ascii'en'This%20is%20even%20more%20;\r\n title*1*=%2A%2A%2Afun%2A%2A%2A%20;\r\n title*2=\"isn't it!\"\r\n").unwrap().1,
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
            ContentType::Multipart
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
