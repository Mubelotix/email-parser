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
        ContentTransferEncoding::SevenBit => {
            for c in value.as_ref().iter() {
                if c >= &127 || c == &0 {
                    return Err(Error::Known("7bit data is containing non-7bit characters"));
                }
            }
            value
        }
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
