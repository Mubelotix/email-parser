use crate::address::*;
use crate::prelude::*;
use std::borrow::Cow;

#[derive(Debug)]
pub struct Email<'a> {
    pub body: Option<Cow<'a, str>>,
    #[cfg(feature = "from")]
    pub from: Vec<Mailbox<'a>>,
}

impl<'a> Email<'a> {
    fn parse(data: &'a [u8]) -> Result<Email<'a>, Error> {
        let (fields, body) = crate::parse_message(data)?;

        #[cfg(feature = "from")]
        let mut from = None;

        for field in fields {
            match field {
                #[cfg(feature = "from")]
                Field::From(mailboxes) => {
                    if from.is_none() {
                        from = Some(mailboxes)
                    } else {
                        return Err(Error::Known("Two from fields"));
                    }
                }
                _ => (),
            }
        }

        Ok(Email {
            body,
            #[cfg(feature = "from")]
            from: from.ok_or(Error::Known("Expected at least one from field"))?,
        })
    }
}

impl<'a> std::convert::TryFrom<&'a [u8]> for Email<'a> {
    type Error = crate::error::Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let mail = Email::parse(b"From: mubelotix@mubelotix.dev\r\n\r\nHey!\r\n").unwrap();
        println!("{:#?}", mail);
    }
}
