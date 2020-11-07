use crate::parsing::time::*;
use crate::prelude::*;

pub fn take_date(input: &[u8]) -> Res<(Option<Day>, Date, Time)> {
    let (input, ()) = tag_no_case(input, b"Date:", b"dATE:")?;
    let (input, date_time) = take_date_time(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, date_time))
}

pub fn take_from(input: &[u8]) -> Res<Vec<(Option<Vec<String>>, (String, String))>> {
    let (input, ()) = tag_no_case(input, b"From:", b"fROM:")?;
    let (input, mailbox_list) = take_mailbox_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox_list))
}

pub fn take_sender(input: &[u8]) -> Res<Mailbox> {
    let (input, ()) = tag_no_case(input, b"Sender:", b"sENDER:")?;
    let (input, mailbox) = take_mailbox(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn take_reply_to(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Reply-To:", b"rEPLY-tO:")?;
    let (input, mailbox) = take_address_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn take_to(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"To:", b"tO:")?;
    let (input, mailbox) = take_address_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn take_cc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Cc:", b"cC:")?;
    let (input, mailbox) = take_address_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn take_bcc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Bcc:", b"bCC:")?;
    let (input, mailbox) = if let Ok((input, list)) = take_address_list(input) {
        (input, list)
    } else if let Ok((input, _cfws)) = take_cfws(input) {
        (input, Vec::new())
    } else {
        return Err(Error::Known("Invalid bcc field"));
    };
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn take_message_id(input: &[u8]) -> Res<(String, String)> {
    let (input, ()) = tag_no_case(input, b"Message-ID:", b"mESSAGE-id:")?;
    let (input, id) = crate::parsing::address::take_message_id(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, id))
}

pub fn take_in_reply_to(input: &[u8]) -> Res<Vec<(String, String)>> {
    let (input, ()) = tag_no_case(input, b"In-Reply-To:", b"iN-rEPLY-tO:")?;
    let (input, ids) = take_many1(input, crate::parsing::address::take_message_id)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, ids))
}

pub fn take_references(input: &[u8]) -> Res<Vec<(String, String)>> {
    let (input, ()) = tag_no_case(input, b"References:", b"rEFERENCES:")?;
    let (input, ids) = take_many1(input, crate::parsing::address::take_message_id)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, ids))
}

pub fn take_subject(input: &[u8]) -> Res<String> {
    let (input, ()) = tag_no_case(input, b"Subject:", b"sUBJECT:")?;
    let (input, subject) = take_unstructured(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, subject))
}

pub fn take_comments(input: &[u8]) -> Res<String> {
    let (input, ()) = tag_no_case(input, b"Comments:", b"cOMMENTS:")?;
    let (input, comments) = take_unstructured(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, comments))
}

pub fn take_keywords(input: &[u8]) -> Res<Vec<Vec<String>>> {
    let (input, ()) = tag_no_case(input, b"Keywords:", b"kEYWORDS:")?;

    let mut keywords = Vec::new();
    let (mut input, first_keyword) = take_phrase(input)?;
    keywords.push(first_keyword);

    while let Ok((new_input, new_keyword)) = take_prefixed(input, take_phrase, ",") {
        input = new_input;
        keywords.push(new_keyword);
    }

    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, keywords))
}

pub fn take_resent_date(input: &[u8]) -> Res<(Option<Day>, Date, Time)> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, date) = take_date(input)?;

    Ok((input, date))
}

pub fn take_resent_from(input: &[u8]) -> Res<Vec<(Option<Vec<String>>, (String, String))>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, from) = take_from(input)?;

    Ok((input, from))
}

pub fn take_resent_sender(input: &[u8]) -> Res<Mailbox> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, sender) = take_sender(input)?;

    Ok((input, sender))
}

pub fn take_resent_to(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, to) = take_to(input)?;

    Ok((input, to))
}

pub fn take_resent_cc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, cc) = take_cc(input)?;

    Ok((input, cc))
}

pub fn take_resent_bcc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, bcc) = take_bcc(input)?;

    Ok((input, bcc))
}

pub fn take_return_path(input: &[u8]) -> Res<Option<(String, String)>> {
    fn take_empty_path(input: &[u8]) -> Res<()> {
        let (input, _cfws) = optional(input, take_cfws);
        let (input, ()) = tag(input, b"<")?;
        let (input, _cfws) = optional(input, take_cfws);
        let (input, ()) = tag(input, b">")?;
        let (input, _cfws) = optional(input, take_cfws);
        Ok((input, ()))
    }

    let (input, ()) = tag_no_case(input, b"Return-Path:", b"rETURN-pATH:")?;
    let (input, addr) = match_parsers(
        input,
        &mut [
            (|i| take_angle_addr(i).map(|(i, v)| (i, Some(v))))
                as fn(input: &[u8]) -> Res<Option<(String, String)>>,
            (|i| take_empty_path(i).map(|(i, _)| (i, None)))
                as fn(input: &[u8]) -> Res<Option<(String, String)>>,
        ][..],
    )?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, addr))
}

#[derive(Debug)]
pub enum ReceivedToken<'a> {
    Word(String<'a>),
    Addr((String<'a>, String<'a>)),
    Domain(String<'a>),
}

pub fn take_received(input: &[u8]) -> Res<(Vec<ReceivedToken>, (Option<Day>, Date, Time))> {
    let (input, ()) = tag_no_case(input, b"Received:", b"rECEIVED:")?;
    let (input, received_tokens) = take_many(input, |input| {
        if let Ok((word_input, word)) = take_word(input) {
            if let Ok((domain_input, domain)) = take_domain(input) {
                if domain.len() > word.len() {
                    return Ok((domain_input, ReceivedToken::Domain(domain)));
                }
            }
            Ok((word_input, ReceivedToken::Word(word)))
        } else if let Ok((input, addr)) = take_angle_addr(input) {
            Ok((input, ReceivedToken::Addr(addr)))
        } else if let Ok((input, addr)) = take_addr_spec(input) {
            Ok((input, ReceivedToken::Addr(addr)))
        } else if let Ok((input, domain)) = take_domain(input) {
            Ok((input, ReceivedToken::Domain(domain)))
        } else {
            Err(Error::Known("match error"))
        }
    })?;
    let (input, ()) = tag(input, b";")?;
    let (input, date_time) = take_date_time(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (received_tokens, date_time)))
}

pub fn take_trace(
    input: &[u8],
) -> Res<(
    Option<Option<(String, String)>>,
    Vec<(Vec<ReceivedToken>, (Option<Day>, Date, Time))>,
)> {
    let (input, return_path) = optional(input, take_return_path);
    let (input, received) = take_many1(input, take_received)?;

    Ok((input, (return_path, received)))
}

pub fn take_unknown(input: &[u8]) -> Res<(String, String)> {
    let (input, name) = take_while1(input, is_ftext)?;
    let (input, ()) = tag(input, b":")?;
    let (input, value) = take_unstructured(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (name, value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_field() {
        assert_eq!(take_unknown(b"hidden-field:hidden message\r\n").unwrap().1.1, "hidden message");
        assert_eq!(take_unknown(b"hidden-field:hidden message\r\n").unwrap().1.0, "hidden-field");
    }

    #[test]
    fn test_trace() {
        assert!(take_return_path(b"Return-Path:<>\r\n").unwrap().1.is_none());
        assert_eq!(
            take_return_path(b"Return-Path:<mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .unwrap()
                .0,
            "mubelotix"
        );

        assert!(matches!(take_received(b"Received:test<mubelotix@gmail.com>;5 May 2003 18:59:03 +0000\r\n").unwrap().1.0[0], ReceivedToken::Word(_)));
        assert!(matches!(take_received(b"Received:test<mubelotix@gmail.com>;5 May 2003 18:59:03 +0000\r\n").unwrap().1.0[1], ReceivedToken::Addr(_)));
        assert!(matches!(take_received(b"Received:mubelotix.dev;5 May 2003 18:59:03 +0000\r\n").unwrap().1.0[0], ReceivedToken::Domain(_)));

        assert!(take_trace(b"Return-Path:<>\r\nReceived:akala miam miam;5 May 2003 18:59:03 +0000\r\nReceived:mubelotix.dev;5 May 2003 18:59:03 +0000\r\n").unwrap().0.is_empty());
    }

    #[test]
    fn test_resent() {
        assert_eq!(
            take_resent_date(b"Resent-Date:5 May 2003 18:59:03 +0000\r\n")
                .unwrap()
                .1,
            (None, (5, Month::May, 2003), ((18, 59, 3), (true, 0, 0)))
        );
        assert_eq!(take_resent_from(b"Resent-FrOm: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1[0].1.0, "mubelotix");
        assert_eq!(take_resent_sender(b"Resent-sender: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1.1.1, "gmail.com");
        assert!(
            !take_resent_to(b"Resent-To: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .is_empty()
        );
        assert!(
            !take_resent_cc(b"Resent-Cc: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .is_empty()
        );
        assert!(
            !take_resent_bcc(b"Resent-Bcc: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .is_empty()
        );
    }

    #[test]
    fn test_date() {
        assert_eq!(
            take_date(b"Date:5 May 2003 18:59:03 +0000\r\n").unwrap().1,
            (None, (5, Month::May, 2003), ((18, 59, 3), (true, 0, 0)))
        );
    }

    #[test]
    fn test_originators() {
        assert_eq!(take_from(b"FrOm: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1[0].1.0, "mubelotix");
        assert_eq!(take_sender(b"sender: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1.1.1, "gmail.com");
        assert_eq!(
            take_reply_to(b"Reply-to: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .len(),
            1
        );
    }

    #[test]
    fn test_destination() {
        assert!(!take_to(b"To: Mubelotix <mubelotix@gmail.com>\r\n")
            .unwrap()
            .1
            .is_empty());
        assert!(!take_cc(b"Cc: Mubelotix <mubelotix@gmail.com>\r\n")
            .unwrap()
            .1
            .is_empty());
        assert!(!take_bcc(b"Bcc: Mubelotix <mubelotix@gmail.com>\r\n")
            .unwrap()
            .1
            .is_empty());
        assert!(take_bcc(b"Bcc: \r\n \r\n").unwrap().1.is_empty());
    }

    #[test]
    fn test_ids() {
        assert_eq!(take_message_id(b"Message-ID:<556100154@gmail.com>\r\n").unwrap().1.0, "556100154");
        assert_eq!(take_message_id(b"Message-ID:<556100154@gmail.com>\r\n").unwrap().1.1, "gmail.com");

        assert_eq!(
            take_references(b"References:<qzdzdq@qdz.com><dzdzjd@zdzdj.dz>\r\n")
                .unwrap()
                .1
                .len(),
            2
        );

        assert_eq!(
            take_in_reply_to(b"In-Reply-To:<eefes@qzd.fr><52@s.dz><adzd@zd.d>\r\n")
                .unwrap()
                .1
                .len(),
            3
        );
    }

    #[test]
    fn test_informational() {
        assert_eq!(
            take_subject(b"Subject:French school is boring\r\n")
                .unwrap()
                .1,
            "French school is boring"
        );
        assert_eq!(
            take_subject(b"Subject:Folding\r\n is slow\r\n").unwrap().1,
            "Folding is slow"
        );

        assert_eq!(
            take_comments(b"Comments:Rust is great\r\n").unwrap().1,
            "Rust is great"
        );

        assert_eq!(
            take_keywords(b"Keywords:rust parser fast zero copy,email rfc5322\r\n")
                .unwrap()
                .1
                .len(),
            2
        );
    }
}
