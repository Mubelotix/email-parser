use crate::address::*;
use crate::prelude::*;
use std::borrow::Cow;

/// A struct representing a valid RFC 5322 message.
///
/// # Example
///
/// ```
/// # use email_parser::prelude::*;
/// let email = Email::parse(
///     b"\
///     From: Mubelotix <mubelotix@mubelotix.dev>\r\n\
///     Subject:Example Email\r\n\
///     To: Someone <example@example.com>\r\n\
///     Message-id: <6546518945@mubelotix.dev>\r\n\
///     Date: 5 May 2003 18:58:34 +0000\r\n\
///     \r\n\
///     Hey!\r\n",
/// )
/// .unwrap();
///
/// assert_eq!(email.subject.unwrap(), "Example Email");
/// assert_eq!(email.sender.name.unwrap(), vec!["Mubelotix"]);
/// assert_eq!(email.sender.address.local_part, "mubelotix");
/// assert_eq!(email.sender.address.domain, "mubelotix.dev");
/// ```
#[derive(Debug)]
pub struct Email<'a> {
    /// The ASCII text of the body.
    #[cfg(not(feature = "mime"))]
    pub body: Option<Cow<'a, str>>,

    #[cfg(feature = "from")]
    /// The list of authors of the message.\
    /// It's **not** the identity of the sender. See the [sender field](#structfield.sender).
    pub from: Vec<Mailbox<'a>>,

    #[cfg(feature = "sender")]
    /// The mailbox of the agent responsible for the actual transmission of the message.\
    /// Do not mix up with the [from field](#structfield.from) that contains the list of authors.\
    /// When there is only one author, this field can be omitted, and its value is inferred. Otherwise, an explicit value is required.
    pub sender: Mailbox<'a>,

    #[cfg(feature = "subject")]
    /// A short optional string identifying the topic of the message.
    pub subject: Option<Cow<'a, str>>,

    #[cfg(feature = "date")]
    /// The date and time at which the [sender](#structfield.sender) of the message indicated that the message was complete and ready to enter the mail delivery system.
    /// For instance, this might be the time that a user pushes the "send" or "submit" button in an application program.
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

    #[cfg(feature = "comments")]
    pub comments: Vec<Cow<'a, str>>,

    #[cfg(feature = "keywords")]
    pub keywords: Vec<Vec<Cow<'a, str>>>,

    #[cfg(feature = "trace")]
    pub trace: Vec<(
        Option<Option<EmailAddress<'a>>>,
        Vec<(Vec<crate::parsing::fields::ReceivedToken<'a>>, DateTime)>,
        Vec<crate::parsing::fields::TraceField<'a>>,
    )>,

    #[cfg(feature = "mime")]
    pub mime_entity: RawEntity<'a>,

    /// The list of unrecognized fields.\
    /// Each field is stored as a `(name, value)` tuple.
    pub unknown_fields: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> Email<'a> {
    /// Parse an email.
    pub fn parse(data: &'a [u8]) -> Result<Email<'a>, Error> {
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
        #[cfg(feature = "mime")]
        let mut mime_version = None;
        #[cfg(feature = "mime")]
        let mut content_type = None;
        #[cfg(feature = "mime")]
        let mut content_transfer_encoding = None;
        #[cfg(feature = "mime")]
        let mut content_id = None;
        #[cfg(feature = "mime")]
        let mut content_description = None;

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
                        return Err(Error::Known("Multiple date fields"));
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
                #[cfg(feature = "comments")]
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
                #[cfg(feature = "mime")]
                Field::MimeVersion(major, minor) => {
                    if mime_version.is_none() {
                        mime_version = Some((major, minor))
                    } else {
                        return Err(Error::Known("Multiple mime_version fields"));
                    }
                }
                #[cfg(feature = "mime")]
                Field::ContentType {
                    mime_type,
                    subtype,
                    parameters,
                } => {
                    if content_type.is_none() {
                        content_type = Some((mime_type, subtype, parameters))
                    } else {
                        return Err(Error::Known("Multiple content_type fields"));
                    }
                }
                #[cfg(feature = "mime")]
                Field::ContentTransferEncoding(encoding) => {
                    if content_transfer_encoding.is_none() {
                        content_transfer_encoding = Some(encoding)
                    } else {
                        return Err(Error::Known("Multiple content_transfer_encoding fields"));
                    }
                }
                #[cfg(feature = "mime")]
                Field::ContentId(id) => {
                    if content_id.is_none() {
                        content_id = Some(id)
                    } else {
                        return Err(Error::Known("Multiple content_id fields"));
                    }
                }
                #[cfg(feature = "mime")]
                Field::ContentDescription(description) => {
                    if content_description.is_none() {
                        content_description = Some(description)
                    } else {
                        return Err(Error::Known("Multiple content_description fields"));
                    }
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

        #[cfg(feature = "mime")]
        let content_type = match content_type {
            Some(content_type) => content_type,
            None => (
                MimeType::Text,
                Cow::Borrowed("plain"),
                vec![(Cow::Borrowed("charset"), Cow::Borrowed("us-ascii"))]
                    .into_iter()
                    .collect(),
            ),
        };
        #[cfg(feature = "mime")]
        let mut content_transfer_encoding = match content_transfer_encoding {
            Some(content_transfer_encoding) => content_transfer_encoding,
            None => ContentTransferEncoding::SevenBit,
        };
        #[cfg(feature = "mime")]
        if content_type.0.is_composite_type()
            && content_transfer_encoding != ContentTransferEncoding::SevenBit
            && content_transfer_encoding != ContentTransferEncoding::HeightBit
            && content_transfer_encoding != ContentTransferEncoding::Binary
        {
            content_transfer_encoding = ContentTransferEncoding::SevenBit;
        }
        #[cfg(feature = "mime")]
        let body = if let Some(body) = body {
            Some(crate::parsing::mime::entity::decode_value(
                Cow::Borrowed(body),
                content_transfer_encoding,
            )?)
        } else {
            None
        };

        Ok(Email {
            #[cfg(not(feature = "mime"))]
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
            #[cfg(feature = "comments")]
            comments,
            #[cfg(feature = "keywords")]
            keywords,
            #[cfg(feature = "mime")]
            mime_entity: RawEntity {
                mime_type: content_type.0,
                subtype: content_type.1,
                description: content_description,
                id: content_id,
                parameters: content_type.2,
                value: body.unwrap_or(Cow::Borrowed(b"")),
            },
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
    fn test_full_email() {
        //println!("{:?}", Email::parse(include_bytes!("../mail.txt")).unwrap().mime_entity.parse().unwrap());
    }

    #[test]
    fn test_field_number() {
        assert!(Email::parse(
            // missing date
            b"\
            From: Mubelotix <mubelotix@mubelotix.dev>\r\n\
            \r\n\
            Hey!\r\n",
        )
        .is_err());

        assert!(Email::parse(
            // 2 date fields
            b"\
            From: Mubelotix <mubelotix@mubelotix.dev>\r\n\
            Date: 5 May 2003 18:58:34 +0000\r\n\
            Date: 6 May 2003 18:58:34 +0000\r\n\
            \r\n\
            Hey!\r\n",
        )
        .is_err());

        assert!(Email::parse(
            // missing from
            b"\
            Date: 5 May 2003 18:58:34 +0000\r\n\
            \r\n\
            Hey!\r\n",
        )
        .is_err());

        assert!(Email::parse(
            // 2 from fields
            b"\
            From: Mubelotix <mubelotix@mubelotix.dev>\r\n\
            From: Someone <jack@gmail.com>\r\n\
            Date: 5 May 2003 18:58:34 +0000\r\n\
            \r\n\
            Hey!\r\n",
        )
        .is_err());
    }
}
