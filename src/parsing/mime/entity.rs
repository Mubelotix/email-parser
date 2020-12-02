use crate::{parsing::fields::unknown, prelude::*};
use std::borrow::Cow;
use std::collections::HashMap;

pub fn entity(mut input: Cow<[u8]>) -> Result<Entity, Error> {
    let (new_input, (encoding, mime_type, subtype, parameters)) =
        header_part(unsafe { &*(input.as_ref() as *const [u8]) })?;
    input = match input {
        Cow::Borrowed(input) => Cow::Borrowed(&input[input.len() - new_input.len()..]),
        Cow::Owned(input) => Cow::Owned(input[input.len() - new_input.len()..].to_owned()),
    };
    let entity = body_part(input, encoding, mime_type, subtype, parameters)?;

    Ok(entity)
}

pub fn header_part(
    mut input: &[u8],
) -> Res<(
    ContentTransferEncoding,
    MimeType,
    Cow<str>,
    HashMap<Cow<str>, Cow<str>>,
)> {
    let mut encoding = None;
    let mut mime_type = None;
    loop {
        if let Ok((new_input, content_transfer_encoding)) = content_transfer_encoding(input) {
            input = new_input;
            encoding = Some(content_transfer_encoding);
        } else if let Ok((new_input, content_type)) = content_type(input) {
            input = new_input;
            mime_type = Some(content_type);
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
        return Ok((input, (encoding, mime_type, subtype, parameters)));
    }

    let (input, _) = tag(&input, b"\r\n")?;

    Ok((input, (encoding, mime_type, subtype, parameters)))
}

pub fn body_part<'a>(
    input: Cow<'a, [u8]>,
    encoding: ContentTransferEncoding,
    mime_type: MimeType<'a>,
    subtype: Cow<'a, str>,
    parameters: HashMap<Cow<str>, Cow<str>>,
) -> Result<Entity<'a>, Error> {
    let value: Cow<[u8]> = match encoding {
        ContentTransferEncoding::Base64 => {
            Cow::Owned(super::base64::decode_base64(input.to_vec())?)
        }
        ContentTransferEncoding::SevenBit => {
            for c in input.as_ref().iter() {
                if c >= &127 || c == &0 {
                    return Err(Error::Known("Invalid 7bit"));
                }
            }
            input
        }
        ContentTransferEncoding::HeightBit => input,
        ContentTransferEncoding::QuotedPrintable => {
            Cow::Owned(super::quoted_printables::decode_qp(input.to_vec()))
        }
        ContentTransferEncoding::Other(_) => {
            return Err(Error::Known("Unknown format"));
        }
        ContentTransferEncoding::Binary => {
            return Err(Error::Known("Unimplemented binary"));
        }
    };

    if mime_type == MimeType::Multipart {
        if let Cow::Borrowed(value) = value {
            return Ok(Entity::Multipart(super::multipart::parse_multipart(
                value, parameters,
            )?));
        }
    }

    if mime_type == MimeType::Text {
        // Fixme: handle charset
        match value {
            Cow::Borrowed(value) => {
                return Ok(Entity::Text {
                    subtype,
                    value: Cow::Borrowed(
                        std::str::from_utf8(value).map_err(|_| Error::Known("Not utf8"))?,
                    ),
                })
            }
            Cow::Owned(value) => {
                return Ok(Entity::Text {
                    subtype,
                    value: Cow::Owned(
                        String::from_utf8(value).map_err(|_| Error::Known("Not utf8"))?,
                    ),
                })
            }
        }
    }

    Ok(Entity::Unknown {
        mime_type,
        subtype,
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_test() {
        println!("{:?}", entity(Cow::Borrowed(b"\r\nText")).unwrap());
        println!("{:?}", entity(Cow::Owned(b"\r\nText".to_vec())).unwrap());
        println!(
            "{:?}",
            entity(Cow::Owned(
                b"Content-type: text/html; charset=utf8\r\n\r\n<p>Text</p>".to_vec()
            ))
            .unwrap()
        );
        println!("{:?}", entity(Cow::Owned(b"Content-type: text/html; charset=utf8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\n<p>Test=C3=A9</p>".to_vec())).unwrap());
        println!("{:?}", entity(Cow::Borrowed(b"Content-type: multipart/alternative; boundary=\"simple boundary\"\r\n\r\nThis is the preamble.  It is to be ignored, though it\r\nis a handy place for composition agents to include an\r\nexplanatory note to non-MIME conformant readers.\r\n\r\n--simple boundary\r\n\r\nThis is implicitly typed plain US-ASCII text.\r\nIt does NOT end with a linebreak.\r\n--simple boundary\r\nContent-type: text/plain; charset=us-ascii\r\n\r\nThis is explicitly typed plain US-ASCII text.\r\nIt DOES end with a linebreak.\r\n\r\n--simple boundary--\r\n\r\nThis is the epilogue.  It is also to be ignored.")).unwrap());
    }
}
