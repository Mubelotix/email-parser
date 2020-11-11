use crate::address::*;
use crate::prelude::*;
use std::borrow::Cow;

#[derive(Debug)]
pub struct Email<'a> {
    pub body: Option<Cow<'a, str>>,
    #[cfg(feature = "from")]
    pub from: Vec<Mailbox<'a>>,
    #[cfg(feature = "subject")]
    pub subject: Option<Cow<'a, str>>,
    #[cfg(feature = "to")]
    pub to: Option<Vec<Address<'a>>>,
    #[cfg(feature = "cc")]
    pub cc: Option<Vec<Address<'a>>>,
    #[cfg(feature = "bcc")]
    pub bcc: Option<Vec<Address<'a>>>,
}

impl<'a> Email<'a> {
    fn parse(data: &'a [u8]) -> Result<Email<'a>, Error> {
        let (fields, body) = crate::parse_message(data)?;

        #[cfg(feature = "from")]
        let mut from = None;
        #[cfg(feature = "subject")]
        let mut subject = None;
        #[cfg(feature = "to")]
        let mut to = None;
        #[cfg(feature = "cc")]
        let mut cc = None;
        #[cfg(feature = "bcc")]
        let mut bcc = None;

        for field in fields {
            match field {
                #[cfg(feature = "from")]
                Field::From(mailboxes) => {
                    if from.is_none() {
                        from = Some(mailboxes)
                    } else {
                        return Err(Error::Known("Multiple from fields"));
                    }
                }
                #[cfg(feature = "subject")]
                Field::Subject(data) => {
                    if subject.is_none() {
                        subject = Some(data)
                    } else {
                        return Err(Error::Known("Multiple subject fields"));
                    }
                },
                #[cfg(feature = "to")]
                Field::To(addresses) => {
                    if to.is_none() {
                        to = Some(addresses)
                    } else {
                        return Err(Error::Known("Multiple to fields"));
                    }
                },
                #[cfg(feature = "cc")]
                Field::Cc(addresses) => {
                    if cc.is_none() {
                        cc = Some(addresses)
                    } else {
                        return Err(Error::Known("Multiple cc fields"));
                    }
                },
                #[cfg(feature = "bcc")]
                Field::Bcc(addresses) => {
                    if bcc.is_none() {
                        bcc = Some(addresses)
                    } else {
                        return Err(Error::Known("Multiple bcc fields"));
                    }
                },
                _ => (),
            }
        }

        Ok(Email {
            body,
            #[cfg(feature = "from")]
            from: from.ok_or(Error::Known("Expected at least one from field"))?,
            #[cfg(feature = "subject")]
            subject,
            #[cfg(feature = "to")]
            to,
            #[cfg(feature = "cc")]
            cc,
            #[cfg(feature = "bcc")]
            bcc,
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
        let mail = Email::parse(b"From: mubelotix@mubelotix.dev\r\nSubject:Testing email\r\n\r\nHey!\r\n").unwrap();
        println!("{:#?}", mail);
    }
}
