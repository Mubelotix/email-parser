use crate::address::*;
use crate::prelude::*;
use std::borrow::Cow;

#[derive(Debug)]
pub struct Email<'a> {
    pub body: Option<Cow<'a, str>>,
    #[cfg(feature = "from")]
    pub from: Vec<Mailbox<'a>>,
    #[cfg(feature = "sender")]
    pub sender: Mailbox<'a>,
    #[cfg(feature = "subject")]
    pub subject: Option<Cow<'a, str>>,
    #[cfg(feature = "date")]
    pub date: DateTime,
    #[cfg(feature = "to")]
    pub to: Option<Vec<Address<'a>>>,
    #[cfg(feature = "cc")]
    pub cc: Option<Vec<Address<'a>>>,
    #[cfg(feature = "bcc")]
    pub bcc: Option<Vec<Address<'a>>>,
    #[cfg(feature = "message-id")]
    pub message_id: Option<(Cow<'a, str>, Cow<'a, str>)>,
    #[cfg(feature = "in-reply-to")]
    pub in_reply_to: Option<Vec<(Cow<'a, str>, Cow<'a, str>)>>,
    #[cfg(feature = "references")]
    pub references: Option<Vec<(Cow<'a, str>, Cow<'a, str>)>>,
    #[cfg(feature = "reply-to")]
    pub reply_to: Option<Vec<Address<'a>>>,
    #[cfg(feature = "trace")]
    pub trace: Vec<(
        Option<Option<EmailAddress<'a>>>,
        Vec<(Vec<crate::parsing::fields::ReceivedToken<'a>>, DateTime)>,
        Vec<crate::parsing::fields::TraceField<'a>>,
    )>,
    pub unknown_fields: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> Email<'a> {
    fn parse(data: &'a [u8]) -> Result<Email<'a>, Error> {
        let (fields, body) = crate::parse_message(data)?;

        #[cfg(feature = "from")]
        let mut from = None;
        #[cfg(feature = "sender")]
        let mut sender = None;
        #[cfg(feature = "subject")]
        let mut subject = None;
        #[cfg(feature = "date")]
        let mut date = None;
        #[cfg(feature = "to")]
        let mut to = None;
        #[cfg(feature = "cc")]
        let mut cc = None;
        #[cfg(feature = "bcc")]
        let mut bcc = None;
        #[cfg(feature = "message-id")]
        let mut message_id = None;
        #[cfg(feature = "in-reply-to")]
        let mut in_reply_to = None;
        #[cfg(feature = "references")]
        let mut references = None;
        #[cfg(feature = "reply-to")]
        let mut reply_to = None;
        #[cfg(feature = "comments")]
        let mut comments = Vec::new();
        #[cfg(feature = "keywords")]
        let mut keywords = Vec::new();
        #[cfg(feature = "trace")]
        let mut trace = Vec::new();

        let mut unknown_fields = Vec::new();

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
                #[cfg(feature = "sender")]
                Field::Sender(mailbox) => {
                    if sender.is_none() {
                        sender = Some(mailbox)
                    } else {
                        return Err(Error::Known("Multiple sender fields"));
                    }
                }
                #[cfg(feature = "subject")]
                Field::Subject(data) => {
                    if subject.is_none() {
                        subject = Some(data)
                    } else {
                        return Err(Error::Known("Multiple subject fields"));
                    }
                }
                #[cfg(feature = "date")]
                Field::Date(data) => {
                    if date.is_none() {
                        date = Some(data)
                    } else {
                        return Err(Error::Known("Multiple subject fields"));
                    }
                }
                #[cfg(feature = "to")]
                Field::To(addresses) => {
                    if to.is_none() {
                        to = Some(addresses)
                    } else {
                        return Err(Error::Known("Multiple to fields"));
                    }
                }
                #[cfg(feature = "cc")]
                Field::Cc(addresses) => {
                    if cc.is_none() {
                        cc = Some(addresses)
                    } else {
                        return Err(Error::Known("Multiple cc fields"));
                    }
                }
                #[cfg(feature = "bcc")]
                Field::Bcc(addresses) => {
                    if bcc.is_none() {
                        bcc = Some(addresses)
                    } else {
                        return Err(Error::Known("Multiple bcc fields"));
                    }
                }
                #[cfg(feature = "message-id")]
                Field::MessageId(id) => {
                    if message_id.is_none() {
                        message_id = Some(id)
                    } else {
                        return Err(Error::Known("Multiple message-id fields"));
                    }
                }
                #[cfg(feature = "in-reply-to")]
                Field::InReplyTo(ids) => {
                    if in_reply_to.is_none() {
                        in_reply_to = Some(ids)
                    } else {
                        return Err(Error::Known("Multiple in-reply-to fields"));
                    }
                }
                #[cfg(feature = "references")]
                Field::References(ids) => {
                    if references.is_none() {
                        references = Some(ids)
                    } else {
                        return Err(Error::Known("Multiple references fields"));
                    }
                }
                #[cfg(feature = "reply-to")]
                Field::ReplyTo(mailboxes) => {
                    if reply_to.is_none() {
                        reply_to = Some(mailboxes)
                    } else {
                        return Err(Error::Known("Multiple reply-to fields"));
                    }
                }
                #[cfg(feature = "reply-to")]
                Field::Comments(data) => comments.push(data),
                #[cfg(feature = "keywords")]
                Field::Keywords(mut data) => {
                    keywords.append(&mut data);
                }
                #[cfg(feature = "trace")]
                Field::Trace {
                    return_path,
                    received,
                    fields,
                } => {
                    trace.push((return_path, received, fields));
                }
                Field::Unknown { name, value } => {
                    unknown_fields.push((name, value));
                }
            }
        }

        #[cfg(feature = "from")]
        let from = from.ok_or(Error::Known("Expected at least one from field"))?;
        #[cfg(feature = "date")]
        let date = date.ok_or(Error::Known("Expected at least one date field"))?;

        #[cfg(feature = "sender")]
        let sender = match sender {
            Some(sender) => sender,
            None => {
                if from.len() == 1 {
                    from[0].clone()
                } else {
                    return Err(Error::Known("Sender field required but missing"));
                }
            }
        };

        Ok(Email {
            body,
            #[cfg(feature = "from")]
            from,
            #[cfg(feature = "sender")]
            sender,
            #[cfg(feature = "subject")]
            subject,
            #[cfg(feature = "date")]
            date,
            #[cfg(feature = "to")]
            to,
            #[cfg(feature = "cc")]
            cc,
            #[cfg(feature = "bcc")]
            bcc,
            #[cfg(feature = "message-id")]
            message_id,
            #[cfg(feature = "in-reply-to")]
            in_reply_to,
            #[cfg(feature = "references")]
            references,
            #[cfg(feature = "reply-to")]
            reply_to,
            #[cfg(feature = "trace")]
            trace,
            unknown_fields,
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
        let mail = Email::parse(b"From: mubelotix@mubelotix.dev\r\nSubject:Testing email\r\nTo: Germanon <germanon@gmail.com>\r\nMessage-id: <6546518945@mubelotix.dev>\r\nDate: 5 May 2003 18:59:03 +0000\r\n\r\nHey!\r\n").unwrap();
        println!("{:#?}", mail);
    }
}
