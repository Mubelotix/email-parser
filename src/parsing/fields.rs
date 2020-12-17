use crate::address::*;
use crate::parsing::time::*;
use crate::prelude::*;
use std::borrow::Cow;
#[cfg(feature = "mime")]
use std::collections::HashMap;

#[derive(Debug)]
pub enum TraceField<'a> {
    Date(DateTime),
    From(Vec<Mailbox<'a>>),
    Sender(Mailbox<'a>),
    To(Vec<Address<'a>>),
    Cc(Vec<Address<'a>>),
    Bcc(Vec<Address<'a>>),
    MessageId((Cow<'a, str>, Cow<'a, str>)),
}

#[derive(Debug)]
pub enum Field<'a> {
    #[cfg(feature = "date")]
    Date(DateTime),
    #[cfg(feature = "from")]
    From(Vec<Mailbox<'a>>),
    #[cfg(feature = "sender")]
    Sender(Mailbox<'a>),
    #[cfg(feature = "reply-to")]
    ReplyTo(Vec<Address<'a>>),
    #[cfg(feature = "to")]
    To(Vec<Address<'a>>),
    #[cfg(feature = "cc")]
    Cc(Vec<Address<'a>>),
    #[cfg(feature = "bcc")]
    Bcc(Vec<Address<'a>>),
    #[cfg(feature = "message-id")]
    MessageId((Cow<'a, str>, Cow<'a, str>)),
    #[cfg(feature = "in-reply-to")]
    InReplyTo(Vec<(Cow<'a, str>, Cow<'a, str>)>),
    #[cfg(feature = "references")]
    References(Vec<(Cow<'a, str>, Cow<'a, str>)>),
    #[cfg(feature = "subject")]
    Subject(Cow<'a, str>),
    #[cfg(feature = "comments")]
    Comments(Cow<'a, str>),
    #[cfg(feature = "keywords")]
    Keywords(Vec<Vec<Cow<'a, str>>>),
    #[cfg(feature = "mime")]
    MimeVersion(u8, u8),
    #[cfg(feature = "mime")]
    ContentType {
        mime_type: MimeType<'a>,
        subtype: Cow<'a, str>,
        parameters: HashMap<Cow<'a, str>, Cow<'a, str>>,
    },
    #[cfg(feature = "mime")]
    ContentTransferEncoding(ContentTransferEncoding<'a>),
    #[cfg(feature = "mime")]
    ContentId((Cow<'a, str>, Cow<'a, str>)),
    #[cfg(feature = "mime")]
    ContentDescription(Cow<'a, str>),
    #[cfg(feature = "content-disposition")]
    ContentDisposition(Disposition<'a>),
    #[cfg(feature = "trace")]
    Trace {
        return_path: Option<Option<EmailAddress<'a>>>,
        received: Vec<(Vec<ReceivedToken<'a>>, DateTime)>,
        fields: Vec<TraceField<'a>>,
    },
    Unknown {
        name: &'a str,
        value: Cow<'a, str>,
    },
}

pub fn fields(mut input: &[u8]) -> Res<Vec<Field>> {
    let mut fields: Vec<Field> = Vec::new();

    #[cfg(feature = "trace")]
    while let Ok((new_input, trace)) = trace(input) {
        input = new_input;
        let mut trace_fields = Vec::new();

        while let Ok((new_input, new_result)) = match_parsers(
            input,
            &mut [
                |i| resent_date(i).map(|(i, v)| (i, TraceField::Date(v))),
                |i| resent_from(i).map(|(i, v)| (i, TraceField::From(v))),
                |i| resent_sender(i).map(|(i, v)| (i, TraceField::Sender(v))),
                |i| resent_to(i).map(|(i, v)| (i, TraceField::To(v))),
                |i| resent_cc(i).map(|(i, v)| (i, TraceField::Cc(v))),
                |i| resent_bcc(i).map(|(i, v)| (i, TraceField::Bcc(v))),
                |i| resent_message_id(i).map(|(i, v)| (i, TraceField::MessageId(v))),
            ][..],
        ) {
            input = new_input;
            trace_fields.push(new_result);
        }

        // TODO optional fields

        fields.push(Field::Trace {
            return_path: trace.0,
            received: trace.1,
            fields: trace_fields,
        });
    }

    while let Ok((new_input, field)) = match_parsers(
        input,
        &mut [
            #[cfg(feature = "date")]
            |i| date(i).map(|(i, v)| (i, Field::Date(v))),
            #[cfg(feature = "from")]
            |i| from(i).map(|(i, v)| (i, Field::From(v))),
            #[cfg(feature = "sender")]
            |i| sender(i).map(|(i, v)| (i, Field::Sender(v))),
            #[cfg(feature = "reply-to")]
            |i| reply_to(i).map(|(i, v)| (i, Field::ReplyTo(v))),
            #[cfg(feature = "to")]
            |i| to(i).map(|(i, v)| (i, Field::To(v))),
            #[cfg(feature = "cc")]
            |i| cc(i).map(|(i, v)| (i, Field::Cc(v))),
            #[cfg(feature = "bcc")]
            |i| bcc(i).map(|(i, v)| (i, Field::Bcc(v))),
            #[cfg(feature = "message-id")]
            |i| message_id(i).map(|(i, v)| (i, Field::MessageId(v))),
            #[cfg(feature = "in-reply-to")]
            |i| in_reply_to(i).map(|(i, v)| (i, Field::InReplyTo(v))),
            #[cfg(feature = "references")]
            |i| references(i).map(|(i, v)| (i, Field::References(v))),
            #[cfg(feature = "subject")]
            |i| subject(i).map(|(i, v)| (i, Field::Subject(v))),
            #[cfg(feature = "comments")]
            |i| comments(i).map(|(i, v)| (i, Field::Comments(v))),
            #[cfg(feature = "mime")]
            |i| mime_version(i).map(|(i, (mj, mn))| (i, Field::MimeVersion(mj, mn))),
            #[cfg(feature = "mime")]
            |i| {
                content_type(i).map(|(i, (t, st, p))| {
                    (
                        i,
                        Field::ContentType {
                            mime_type: t,
                            subtype: st,
                            parameters: p,
                        },
                    )
                })
            },
            #[cfg(feature = "mime")]
            |i| content_transfer_encoding(i).map(|(i, e)| (i, Field::ContentTransferEncoding(e))),
            #[cfg(feature = "mime")]
            |i| content_id(i).map(|(i, v)| (i, Field::ContentId(v))),
            #[cfg(feature = "mime")]
            |i| content_description(i).map(|(i, d)| (i, Field::ContentDescription(d))),
            #[cfg(feature = "content-disposition")]
            |i| content_disposition(i).map(|(i, d)| (i, Field::ContentDisposition(d))),
            #[cfg(feature = "keywords")]
            |i| keywords(i).map(|(i, v)| (i, Field::Keywords(v))),
            |i| unknown(i).map(|(i, (name, value))| (i, Field::Unknown { name, value })),
        ][..],
    ) {
        input = new_input;
        fields.push(field);
    }

    Ok((input, fields))
}

pub fn date(input: &[u8]) -> Res<DateTime> {
    let (input, ()) = tag_no_case(input, b"Date:", b"dATE:")?;
    let (input, date_time) = date_time(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, date_time))
}

pub fn from(input: &[u8]) -> Res<Vec<Mailbox>> {
    let (input, ()) = tag_no_case(input, b"From:", b"fROM:")?;
    let (input, mailbox_list) = mailbox_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox_list))
}

pub fn sender(input: &[u8]) -> Res<Mailbox> {
    let (input, ()) = tag_no_case(input, b"Sender:", b"sENDER:")?;
    let (input, mailbox) = mailbox(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn reply_to(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Reply-To:", b"rEPLY-tO:")?;
    let (input, mailbox) = address_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn to(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"To:", b"tO:")?;
    let (input, mailbox) = address_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn cc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Cc:", b"cC:")?;
    let (input, mailbox) = address_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn bcc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Bcc:", b"bCC:")?;
    let (input, mailbox) = if let Ok((input, list)) = address_list(input) {
        (input, list)
    } else if let Ok((input, _cfws)) = cfws(input) {
        (input, Vec::new())
    } else {
        return Err(Error::Unknown("Invalid bcc field"));
    };
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn message_id(input: &[u8]) -> Res<(Cow<str>, Cow<str>)> {
    let (input, ()) = tag_no_case(input, b"Message-ID:", b"mESSAGE-id:")?;
    let (input, id) = crate::parsing::address::message_id(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, id))
}

pub fn in_reply_to(input: &[u8]) -> Res<Vec<(Cow<str>, Cow<str>)>> {
    let (input, ()) = tag_no_case(input, b"In-Reply-To:", b"iN-rEPLY-tO:")?;
    let (input, ids) = many1(input, crate::parsing::address::message_id)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, ids))
}

pub fn references(input: &[u8]) -> Res<Vec<(Cow<str>, Cow<str>)>> {
    let (input, ()) = tag_no_case(input, b"References:", b"rEFERENCES:")?;
    let (input, ids) = many1(input, crate::parsing::address::message_id)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, ids))
}

pub fn subject(input: &[u8]) -> Res<Cow<str>> {
    let (input, ()) = tag_no_case(input, b"Subject:", b"sUBJECT:")?;
    #[cfg(not(feature = "mime"))]
    let (input, subject) = unstructured(input)?;
    #[cfg(feature = "mime")]
    let (input, subject) = mime_unstructured(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, subject))
}

pub fn comments(input: &[u8]) -> Res<Cow<str>> {
    let (input, ()) = tag_no_case(input, b"Comments:", b"cOMMENTS:")?;
    #[cfg(not(feature = "mime"))]
    let (input, comments) = unstructured(input)?;
    #[cfg(feature = "mime")]
    let (input, comments) = mime_unstructured(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, comments))
}

pub fn keywords(input: &[u8]) -> Res<Vec<Vec<Cow<str>>>> {
    let (input, ()) = tag_no_case(input, b"Keywords:", b"kEYWORDS:")?;

    let mut keywords = Vec::new();
    let (mut input, first_keyword) = phrase(input)?;
    keywords.push(first_keyword);

    while let Ok((new_input, new_keyword)) = prefixed(input, phrase, ",") {
        input = new_input;
        keywords.push(new_keyword);
    }

    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, keywords))
}

pub fn resent_date(input: &[u8]) -> Res<DateTime> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, date) = date(input)?;

    Ok((input, date))
}

pub fn resent_from(input: &[u8]) -> Res<Vec<Mailbox>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, from) = from(input)?;

    Ok((input, from))
}

pub fn resent_sender(input: &[u8]) -> Res<Mailbox> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, sender) = sender(input)?;

    Ok((input, sender))
}

pub fn resent_to(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, to) = to(input)?;

    Ok((input, to))
}

pub fn resent_cc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, cc) = cc(input)?;

    Ok((input, cc))
}

pub fn resent_bcc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, bcc) = bcc(input)?;

    Ok((input, bcc))
}

pub fn resent_message_id(input: &[u8]) -> Res<(Cow<str>, Cow<str>)> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, id) = message_id(input)?;

    Ok((input, id))
}

pub fn return_path(input: &[u8]) -> Res<Option<EmailAddress>> {
    fn empty_path(input: &[u8]) -> Res<()> {
        let (input, _cfws) = optional(input, cfws);
        let (input, ()) = tag(input, b"<")?;
        let (input, _cfws) = optional(input, cfws);
        let (input, ()) = tag(input, b">")?;
        let (input, _cfws) = optional(input, cfws);
        Ok((input, ()))
    }

    let (input, ()) = tag_no_case(input, b"Return-Path:", b"rETURN-pATH:")?;
    let (input, addr) = match_parsers(
        input,
        &mut [
            (|i| angle_addr(i).map(|(i, v)| (i, Some(v))))
                as fn(input: &[u8]) -> Res<Option<EmailAddress>>,
            (|i| empty_path(i).map(|(i, _)| (i, None)))
                as fn(input: &[u8]) -> Res<Option<EmailAddress>>,
        ][..],
    )?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, addr))
}

#[derive(Debug)]
pub enum ReceivedToken<'a> {
    Word(Cow<'a, str>),
    Addr(EmailAddress<'a>),
    Domain(Cow<'a, str>),
}

pub fn received(input: &[u8]) -> Res<(Vec<ReceivedToken>, DateTime)> {
    let (input, ()) = tag_no_case(input, b"Received:", b"rECEIVED:")?;
    let (input, received_tokens) = many(input, |input| {
        if let Ok((word_input, word)) = word(input) {
            if let Ok((domain_input, domain)) = domain(input) {
                if domain.len() > word.len() {
                    return Ok((domain_input, ReceivedToken::Domain(domain)));
                }
            }
            Ok((word_input, ReceivedToken::Word(word)))
        } else if let Ok((input, addr)) = angle_addr(input) {
            Ok((input, ReceivedToken::Addr(addr)))
        } else if let Ok((input, addr)) = addr_spec(input) {
            Ok((input, ReceivedToken::Addr(addr)))
        } else if let Ok((input, domain)) = domain(input) {
            Ok((input, ReceivedToken::Domain(domain)))
        } else {
            Err(Error::Unknown("match error"))
        }
    })?;
    let (input, ()) = tag(input, b";")?;
    let (input, date_time) = date_time(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (received_tokens, date_time)))
}

pub fn trace(
    input: &[u8],
) -> Res<(
    Option<Option<EmailAddress>>,
    Vec<(Vec<ReceivedToken>, DateTime)>,
)> {
    let (input, return_path) = optional(input, return_path);
    let (input, received) = many1(input, received)?;

    Ok((input, (return_path, received)))
}

pub fn unknown(input: &[u8]) -> Res<(&str, Cow<str>)> {
    let (input, name) = take_while1(input, is_ftext)?;
    let (input, ()) = tag(input, b":")?;
    #[cfg(not(feature = "unrecognized-headers"))]
    let (input, value) = unstructured(input)?;
    #[cfg(feature = "unrecognized-headers")]
    let (input, value) = mime_unstructured(input)?;

    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (name, value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fields() {
        assert!(fields(
            b"To: Mubelotix <mubelotix@gmail.com>\r\nFrOm: Mubelotix <mubelotix@gmail.com>\r\n"
        )
        .unwrap()
        .0
        .is_empty());
        //println!("{:#?}", fields(include_bytes!("../../mail.txt")).unwrap().1);
    }

    #[test]
    fn test_unknown_field() {
        assert_eq!(
            unknown(b"hidden-field:hidden message\r\n").unwrap().1 .1,
            "hidden message"
        );
        assert_eq!(
            unknown(b"hidden-field:hidden message\r\n").unwrap().1 .0,
            "hidden-field"
        );
    }

    #[test]
    fn test_trace() {
        assert!(return_path(b"Return-Path:<>\r\n").unwrap().1.is_none());
        assert_eq!(
            return_path(b"Return-Path:<mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .unwrap()
                .local_part,
            "mubelotix"
        );

        assert!(matches!(
            received(b"Received:test<mubelotix@gmail.com>;5 May 2003 18:59:03 +0000\r\n")
                .unwrap()
                .1
                 .0[0],
            ReceivedToken::Word(_)
        ));
        assert!(matches!(
            received(b"Received:test<mubelotix@gmail.com>;5 May 2003 18:59:03 +0000\r\n")
                .unwrap()
                .1
                 .0[1],
            ReceivedToken::Addr(_)
        ));
        assert!(matches!(
            received(b"Received:mubelotix.dev;5 May 2003 18:59:03 +0000\r\n")
                .unwrap()
                .1
                 .0[0],
            ReceivedToken::Domain(_)
        ));

        assert!(trace(b"Return-Path:<>\r\nReceived:akala miam miam;5 May 2003 18:59:03 +0000\r\nReceived:mubelotix.dev;5 May 2003 18:59:03 +0000\r\n").unwrap().0.is_empty());
    }

    #[test]
    fn test_resent() {
        assert!(resent_date(b"Resent-Date:5 May 2003 18:59:03 +0000\r\n").is_ok());
        assert_eq!(
            resent_from(b"Resent-FrOm: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1[0]
                .address
                .local_part,
            "mubelotix"
        );
        assert_eq!(
            resent_sender(b"Resent-sender: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .address
                .domain,
            "gmail.com"
        );
        assert!(
            !resent_to(b"Resent-To: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .is_empty()
        );
        assert!(
            !resent_cc(b"Resent-Cc: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .is_empty()
        );
        assert!(
            !resent_bcc(b"Resent-Bcc: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .is_empty()
        );
    }

    #[test]
    fn test_date() {
        assert!(date(b"Date:5 May 2003 18:59:03 +0000\r\n").is_ok());
    }

    #[test]
    fn test_originators() {
        assert_eq!(
            from(b"FrOm: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1[0]
                .address
                .local_part,
            "mubelotix"
        );
        assert_eq!(
            sender(b"sender: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .address
                .domain,
            "gmail.com"
        );
        assert_eq!(
            reply_to(b"Reply-to: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .len(),
            1
        );
    }

    #[test]
    fn test_destination() {
        assert!(!to(b"To: Mubelotix <mubelotix@gmail.com>\r\n")
            .unwrap()
            .1
            .is_empty());
        assert!(!cc(b"Cc: Mubelotix <mubelotix@gmail.com>\r\n")
            .unwrap()
            .1
            .is_empty());
        assert!(!bcc(b"Bcc: Mubelotix <mubelotix@gmail.com>\r\n")
            .unwrap()
            .1
            .is_empty());
        assert!(bcc(b"Bcc: \r\n \r\n").unwrap().1.is_empty());
    }

    #[test]
    fn test_ids() {
        assert_eq!(
            message_id(b"Message-ID:<556100154@gmail.com>\r\n")
                .unwrap()
                .1
                 .0,
            "556100154"
        );
        assert_eq!(
            message_id(b"Message-ID:<556100154@gmail.com>\r\n")
                .unwrap()
                .1
                 .1,
            "gmail.com"
        );

        assert_eq!(
            references(b"References:<qzdzdq@qdz.com><dzdzjd@zdzdj.dz>\r\n")
                .unwrap()
                .1
                .len(),
            2
        );

        assert_eq!(
            in_reply_to(b"In-Reply-To:<eefes@qzd.fr><52@s.dz><adzd@zd.d>\r\n")
                .unwrap()
                .1
                .len(),
            3
        );
    }

    #[test]
    fn test_informational() {
        assert_eq!(
            subject(b"Subject:French school is boring\r\n").unwrap().1,
            "French school is boring"
        );
        assert_eq!(
            subject(b"Subject:Folding\r\n is slow\r\n").unwrap().1,
            "Folding is slow"
        );

        assert_eq!(
            comments(b"Comments:Rust is great\r\n").unwrap().1,
            "Rust is great"
        );

        assert_eq!(
            keywords(b"Keywords:rust parser fast zero copy,email rfc5322\r\n")
                .unwrap()
                .1
                .len(),
            2
        );
    }

    #[test]
    #[cfg(all(feature = "mime", feature = "unrecognized-headers"))]
    fn test_mime_encoding() {
        assert_eq!(
            subject(b"Subject: =?UTF-8?B?8J+OiEJpcnRoZGF5IEdpdmVhd2F58J+OiA==?= Win free stickers\r\n from daily.dev =?UTF-8?B?8J+MiA==?=\r\n").unwrap().1,
            " 🎈Birthday Giveaway🎈 Win free stickers from daily.dev 🌈"
        );

        assert_eq!(
            comments(b"Comments: =?ISO-8859-1?B?SWYgeW91IGNhbiByZWFkIHRoaXMgeW8=?=\r\n =?ISO-8859-2?B?dSB1bmRlcnN0YW5kIHRoZSBleGFtcGxlLg==?=\r\n").unwrap().1,
            " If you can read this you understand the example."
        );

        assert_eq!(
            from(b"From: =?US-ASCII?Q?Keith_Moore?= <moore@cs.utk.edu>\r\n")
                .unwrap()
                .1[0]
                .name
                .as_ref()
                .unwrap()[0],
            "Keith Moore"
        );

        assert_eq!(
            unknown(b"X-SG-EID:\r\n =?us-ascii?Q?t3vk5cTFE=2FYEGeQ8h3SwrnzIAGc=2F+ADymlys=2FfRFW4Zjpt=2F3MuaO9JNHS2enYQ?=\r\n =?us-ascii?Q?Jsv0=2FpYrPem+YssHetKlrE5nJnOfr=2FYdJOyJFf8?=\r\n =?us-ascii?Q?3mRuMRE9KGu=2F5O75=2FwwN6dG14nuP4SyMIZwbMdG?=\r\n =?us-ascii?Q?vXmM2kgcM=2FOalKeT03BMp4YCg9h1LhkV6PZEoHB?=\r\n =?us-ascii?Q?d4tcAvNZQqLaA4ykI1EpNxKVVyZXVWqTp2uisdf?=\r\n =?us-ascii?Q?HB=2F6BKcIs+XSDNeakQqmn=2FwAqOk78AvtRB5LnNL?=\r\n =?us-ascii?Q?lz3oRXlMZbdFgRH+KAyLQ=3D=3D?=\r\n").unwrap().1.1,
            " t3vk5cTFE/YEGeQ8h3SwrnzIAGc/+ADymlys/fRFW4Zjpt/3MuaO9JNHS2enYQJsv0/pYrPem+YssHetKlrE5nJnOfr/YdJOyJFf83mRuMRE9KGu/5O75/wwN6dG14nuP4SyMIZwbMdGvXmM2kgcM/OalKeT03BMp4YCg9h1LhkV6PZEoHBd4tcAvNZQqLaA4ykI1EpNxKVVyZXVWqTp2uisdfHB/6BKcIs+XSDNeakQqmn/wAqOk78AvtRB5LnNLlz3oRXlMZbdFgRH+KAyLQ=="
        );
    }
}
