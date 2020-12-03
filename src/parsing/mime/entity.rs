use crate::{parsing::fields::unknown, prelude::*};
use std::borrow::Cow;
use std::collections::HashMap;

use super::multipart;

pub fn raw_entity(mut input: Cow<[u8]>) -> Result<RawEntity, Error> {
    let (new_input, (encoding, mime_type, subtype, parameters, id, description)) =
        header_part(unsafe { &*(input.as_ref() as *const [u8]) })?;
    match input {
        Cow::Borrowed(ref mut input) => *input = &input[input.len() - new_input.len()..],
        Cow::Owned(ref mut input) => {
            input.drain(..input.len() - new_input.len());
        }
    };
    let value = decode_value(input, encoding)?;

    Ok(RawEntity {
        mime_type,
        subtype,
        parameters,
        id,
        description,
        value,
    })
}

pub fn entity(raw_entity: RawEntity) -> Result<Entity, Error> {
    if raw_entity.mime_type == MimeType::Multipart {
        match raw_entity.value {
            Cow::Borrowed(value) => {
                return Ok(Entity::Multipart {
                    subtype: raw_entity.subtype,
                    content: multipart::parse_multipart(value, raw_entity.parameters)?,
                })
            }
            Cow::Owned(value) => {
                return Ok(Entity::Multipart {
                    subtype: raw_entity.subtype,
                    content: multipart::parse_multipart_owned(value, raw_entity.parameters)?,
                })
            }
        }
    }

    if raw_entity.mime_type == MimeType::Text {
        use textcode::*;

        let charset = raw_entity
            .parameters
            .get("charset")
            .unwrap_or(&Cow::Borrowed("us-ascii"))
            .to_lowercase();

        let value: Cow<str> = match charset.as_str() {
            "utf-8" | "us-ascii" => match raw_entity.value {
                Cow::Borrowed(value) => Cow::Borrowed(
                    std::str::from_utf8(value)
                        .map_err(|_| Error::Known("Invalid text encoding"))?,
                ),
                Cow::Owned(value) => Cow::Owned(
                    String::from_utf8(value).map_err(|_| Error::Known("Invalid text encoding"))?,
                ),
            },
            "iso-8859-1" => Cow::Owned(iso8859_1::decode_to_string(&raw_entity.value)),
            "iso-8859-2" => Cow::Owned(iso8859_2::decode_to_string(&raw_entity.value)),
            "iso-8859-3" => Cow::Owned(iso8859_3::decode_to_string(&raw_entity.value)),
            "iso-8859-4" => Cow::Owned(iso8859_4::decode_to_string(&raw_entity.value)),
            "iso-8859-5" => Cow::Owned(iso8859_5::decode_to_string(&raw_entity.value)),
            "iso-8859-6" => Cow::Owned(iso8859_6::decode_to_string(&raw_entity.value)),
            "iso-8859-7" => Cow::Owned(iso8859_7::decode_to_string(&raw_entity.value)),
            "iso-8859-8" => Cow::Owned(iso8859_8::decode_to_string(&raw_entity.value)),
            "iso-8859-9" => Cow::Owned(iso8859_9::decode_to_string(&raw_entity.value)),
            "iso-8859-10" => Cow::Owned(iso8859_10::decode_to_string(&raw_entity.value)),
            "iso-8859-11" => Cow::Owned(iso8859_11::decode_to_string(&raw_entity.value)),
            "iso-8859-13" => Cow::Owned(iso8859_13::decode_to_string(&raw_entity.value)),
            "iso-8859-14" => Cow::Owned(iso8859_14::decode_to_string(&raw_entity.value)),
            "iso-8859-15" => Cow::Owned(iso8859_15::decode_to_string(&raw_entity.value)),
            "iso-8859-16" => Cow::Owned(iso8859_16::decode_to_string(&raw_entity.value)),
            "iso-6937" => Cow::Owned(iso6937::decode_to_string(&raw_entity.value)),
            "gb2312" => Cow::Owned(gb2312::decode_to_string(&raw_entity.value)),
            _ => return Ok(Entity::Unknown(raw_entity)),
        };

        return Ok(Entity::Text {
            subtype: raw_entity.subtype,
            value,
        });
    }

    Ok(Entity::Unknown(raw_entity))
}

pub fn header_part(
    mut input: &[u8],
) -> Res<(
    ContentTransferEncoding,
    MimeType,
    Cow<str>,
    HashMap<Cow<str>, Cow<str>>,
    Option<(Cow<str>, Cow<str>)>,
    Option<Cow<str>>,
)> {
    let mut encoding = None;
    let mut mime_type = None;
    let mut id = None;
    let mut description = None;

    loop {
        // FIXME: Should we trigger errors on duplicated fields?
        if let Ok((new_input, content_transfer_encoding)) = content_transfer_encoding(input) {
            input = new_input;
            encoding = Some(content_transfer_encoding);
        } else if let Ok((new_input, content_type)) = content_type(input) {
            input = new_input;
            mime_type = Some(content_type);
        } else if let Ok((new_input, cid)) = content_id(input) {
            input = new_input;
            id = Some(cid);
        } else if let Ok((new_input, cdescription)) = content_description(input) {
            input = new_input;
            description = Some(cdescription);
        } else if let Ok((new_input, _unknown)) = unknown(input) {
            input = new_input;
        } else {
            break;
        }
    }

    let encoding = encoding.unwrap_or(ContentTransferEncoding::SevenBit);
    let (mime_type, subtype, parameters) = mime_type.unwrap_or((
        MimeType::Text,
        Cow::Borrowed("plain"),
        vec![(Cow::Borrowed("charset"), Cow::Borrowed("us-ascii"))]
            .into_iter()
            .collect(),
    ));

    if input.is_empty() {
        return Ok((
            input,
            (encoding, mime_type, subtype, parameters, id, description),
        ));
    }

    let (input, _) = tag(&input, b"\r\n")?;

    Ok((
        input,
        (encoding, mime_type, subtype, parameters, id, description),
    ))
}

pub fn decode_value<'a>(
    value: Cow<'a, [u8]>,
    encoding: ContentTransferEncoding,
) -> Result<Cow<'a, [u8]>, Error> {
    Ok(match encoding {
        ContentTransferEncoding::Base64 => {
            Cow::Owned(super::base64::decode_base64(value.into_owned())?)
        }
        ContentTransferEncoding::SevenBit => value, // No need to check, we have to be tolerant
        ContentTransferEncoding::HeightBit => value,
        ContentTransferEncoding::QuotedPrintable => {
            Cow::Owned(super::quoted_printables::decode_qp(value.into_owned()))
        }
        ContentTransferEncoding::Other(_) => {
            return Err(Error::Known("Unknown format")); // FIXME: Allow user to get this data
        }
        ContentTransferEncoding::Binary => value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_entity_test() {
        println!("{:?}", raw_entity(Cow::Borrowed(b"\r\nText")).unwrap());
        println!(
            "{:?}",
            raw_entity(Cow::Owned(b"\r\nText".to_vec())).unwrap()
        );
        println!(
            "{:?}",
            raw_entity(Cow::Owned(
                b"Content-type: text/html; charset=utf8\r\n\r\n<p>Text</p>".to_vec()
            ))
            .unwrap()
        );
        println!("{:?}", raw_entity(Cow::Owned(b"Content-type: text/html; charset=utf8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\n<p>Test=C3=A9</p>".to_vec())).unwrap());
        println!("{:?}", raw_entity(Cow::Borrowed(b"Content-type: multipart/alternative; boundary=\"simple boundary\"\r\n\r\nThis is the preamble.  It is to be ignored, though it\r\nis a handy place for composition agents to include an\r\nexplanatory note to non-MIME conformant readers.\r\n\r\n--simple boundary\r\n\r\nThis is implicitly typed plain US-ASCII text.\r\nIt does NOT end with a linebreak.\r\n--simple boundary\r\nContent-type: text/plain; charset=us-ascii\r\n\r\nThis is explicitly typed plain US-ASCII text.\r\nIt DOES end with a linebreak.\r\n\r\n--simple boundary--\r\n\r\nThis is the epilogue.  It is also to be ignored.")).unwrap());
    }
}
