use crate::prelude::*;
use crate::parsing::time::*;

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

#[cfg(test)]
mod tests {
    use super::*;

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
