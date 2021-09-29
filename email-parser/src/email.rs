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
    pub unknown_fields: Vec<(&'a str, Cow<'a, str>)>,
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
        #[cfg(feature = "content-disposition")]
        let mut content_disposition = None;

        let mut unknown_fields = Vec::new();

        for field in fields {
            match field {
                #[cfg(feature = "from")]
                Field::From(mailboxes) => {
                    merge_headers(&mut from, mailboxes, "From")?;
                }
                #[cfg(feature = "sender")]
                Field::Sender(mailbox) => {
                    assign_header(&mut sender, mailbox, "Sender")?;
                }
                #[cfg(feature = "subject")]
                Field::Subject(data) => {
                    assign_header(&mut subject, data, "Subject")?;
                }
                #[cfg(feature = "date")]
                Field::Date(data) => {
                    assign_header(&mut date, data, "Date")?;
                }
                #[cfg(feature = "to")]
                Field::To(addresses) => {
                    merge_headers(&mut to, addresses, "To")?;
                }
                #[cfg(feature = "cc")]
                Field::Cc(addresses) => {
                    merge_headers(&mut cc, addresses, "Cc")?;
                }
                #[cfg(feature = "bcc")]
                Field::Bcc(addresses) => {
                    merge_headers(&mut bcc, addresses, "Bcc")?;
                }
                #[cfg(feature = "message-id")]
                Field::MessageId(id) => {
                    assign_header(&mut message_id, id, "Message-ID")?;
                }
                #[cfg(feature = "in-reply-to")]
                Field::InReplyTo(ids) => {
                    merge_headers(&mut in_reply_to, ids, "In-Reply-To")?;
                }
                #[cfg(feature = "references")]
                Field::References(ids) => {
                    merge_headers(&mut references, ids, "References")?;
                }
                #[cfg(feature = "reply-to")]
                Field::ReplyTo(mailboxes) => {
                    merge_headers(&mut reply_to, mailboxes, "Reply-To")?;
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
                    assign_header(&mut mime_version, (major, minor), "Mime-Version")?;
                }
                #[cfg(feature = "mime")]
                Field::ContentType {
                    mime_type,
                    subtype,
                    parameters,
                } => {
                    assign_header(
                        &mut content_type,
                        (mime_type, subtype, parameters),
                        "Content-Type",
                    )?;
                }
                #[cfg(feature = "mime")]
                Field::ContentTransferEncoding(encoding) => {
                    assign_header(
                        &mut content_transfer_encoding,
                        encoding,
                        "Content-Transfer-Encoding",
                    )?;
                }
                #[cfg(feature = "mime")]
                Field::ContentId(id) => {
                    assign_header(&mut content_id, id, "Content-Id")?;
                }
                #[cfg(feature = "mime")]
                Field::ContentDescription(description) => {
                    assign_header(&mut content_description, description, "Content-Description")?;
                }
                #[cfg(feature = "content-disposition")]
                Field::ContentDisposition(disposition) => {
                    assign_header(&mut content_disposition, disposition, "Content-Disposition")?;
                }
                Field::Unknown { name, value } => {
                    unknown_fields.push((name, value));
                }
            }
        }

        #[cfg(feature = "from")]
        let from = from.ok_or(Error::MissingHeader("From"))?;
        #[cfg(feature = "date")]
        let date = date.ok_or(Error::MissingHeader("Date"))?;

        #[cfg(feature = "sender")]
        let sender = match sender {
            Some(sender) => sender,
            None => {
                if from.len() >= 1 {
                    from[0].clone()
                } else {
                    return Err(Error::MissingHeader("Sender"));
                }
            }
        };

        #[cfg(feature = "mime")]
        let (content_type, body) = (
            content_type.unwrap_or((
                ContentType::Text,
                Cow::Borrowed("plain"),
                vec![(Cow::Borrowed("charset"), Cow::Borrowed("us-ascii"))]
                    .into_iter()
                    .collect(),
            )),
            #[allow(unused_variables)]
            if let Some(body) = body {
                #[cfg(feature = "decode-mime-body")]
                {
                    crate::parsing::mime::entity::decode_value(
                        Cow::Borrowed(body),
                        content_transfer_encoding.unwrap_or(ContentTransferEncoding::SevenBit),
                    )
                    .ok()
                }

                #[cfg(not(feature = "decode-mime-body"))]
                {
                    None
                }
            } else {
                None
            },
        );

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
                #[cfg(feature = "content-disposition")]
                disposition: content_disposition,
                value: body.unwrap_or(Cow::Borrowed(b"")),
                additional_headers: Vec::new(),
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

#[allow(unused_variables, unused_mut)]
fn merge_headers<T>(
    existing: &mut Option<Vec<T>>,
    mut new: Vec<T>,
    name: &'static str,
) -> Result<(), Error> {
    #[cfg(not(feature = "allow-duplicate-headers"))]
    if existing.is_some() {
        return Err(Error::DuplicateHeader(name));
    } else {
        *existing = Some(new);
    }

    #[cfg(feature = "allow-duplicate-headers")]
    if let Some(value) = existing.as_mut() {
        let value: &mut Vec<T> = value;
        value.append(&mut new);
    } else {
        *existing = Some(new);
    }
    Ok(())
}

#[allow(unused_variables)]
fn assign_header<T>(existing: &mut Option<T>, new: T, name: &'static str) -> Result<(), Error> {
    #[cfg(not(feature = "allow-duplicate-headers"))]
    if existing.is_some() {
        return Err(Error::DuplicateHeader(name));
    } else {
        *existing = Some(new);
    }

    #[cfg(feature = "allow-duplicate-headers")]
    if existing.is_none() {
        *existing = Some(new);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_full_email() {
        /*let multipart = Email::parse(include_bytes!("../mail.txt")).unwrap().mime_entity.parse().unwrap();
        println!("{:?}", multipart);
        if let Entity::Multipart{content, subtype: _} = multipart {
            for entity in content {
                println!("{:?}", entity.parse().unwrap())
            }
        } else {
            panic!("Failed to parse multipart");
        }*/
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

        #[cfg(not(feature = "allow-duplicate-headers"))]
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

        #[cfg(not(feature = "allow-duplicate-headers"))]
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
