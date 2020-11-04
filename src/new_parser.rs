use crate::prelude::*;




pub mod address {
    use super::*;
    use crate::prelude::*;

    pub type Mailbox<'a> = (Option<Vec<String<'a>>>, (String<'a>, String<'a>));

    pub enum Address<'a> {
        Mailbox(Mailbox<'a>),
        Group((Vec<String<'a>>, Vec<Mailbox<'a>>)),
    }

    #[inline]
    pub fn is_dtext(c: u8) -> bool {
        (c >= 33 && c <= 90) || (c >= 94 && c <= 126)
    }

    pub fn take_message_id(input: &[u8]) -> Res<(String, String)> {
        fn take_no_fold_litteral(input: &[u8]) -> Res<String> {
            let (input, ()) = tag(input, b"[")?;
            let (input, domain) = take_while(input, is_dtext)?;
            let (input, ()) = tag(input, b"]")?;
            Ok((input, domain))
        }

        let (input, _cfws) = optional(input, take_cfws);
        let (input, ()) = tag(input, b"<")?;
        let (input, id_left) = take_dot_atom_text(input)?;
        let (input, ()) = tag(input, b"@")?;
        let (input, id_right) = match_parsers(input, &mut [take_dot_atom_text, take_no_fold_litteral][..])?;
        let (input, ()) = tag(input, b">")?;
        let (input, _cfws) = optional(input, take_cfws);

        Ok((input, (id_left, id_right)))
    }

    pub fn take_addr_spec(input: &[u8]) -> Res<(String, String)> {
        let (input, local_part) = take_local_part(input)?;
        let (input, ()) = tag(input, b"@")?;
        let (input, domain) = take_domain(input)?;
        Ok((input, (local_part, domain)))
    }

    pub fn take_name_addr(input: &[u8]) -> Res<Mailbox> {
        let (input, display_name) = optional(input, take_phrase);
        let (input, _cfws) = optional(input, take_cfws);
        let (input, ()) = tag(input, b"<")?;
        let (input, addr_spec) = take_addr_spec(input)?;
        let (input, ()) = tag(input, b">")?;
        let (input, _cfws) = optional(input, take_cfws);
        Ok((input, (display_name, addr_spec)))
    }

    pub fn take_local_part(input: &[u8]) -> Res<String> {
        match_parsers(input, &mut [take_dot_atom, take_quoted_string][..])
    }

    pub fn take_domain(input: &[u8]) -> Res<String> {
        match_parsers(input, &mut [take_dot_atom, take_domain_literal][..])
    }

    pub fn take_domain_literal(input: &[u8]) -> Res<String> {
        let (input, _cfws) = optional(input, take_cfws);
        let (mut input, ()) = tag(input, b"[")?;
        let mut output = String::Reference(&[]);
        loop {
            let (new_input, _fws) = optional(input, take_fws);
            if let Ok((new_input, text)) = take_while1(new_input, is_dtext) {
                input = new_input;
                //output += fws; should it be added?
                output += text;
            } else {
                break;
            }
        }
        let (input, _fws) = optional(input, take_fws);
        let (input, ()) = tag(input, b"]")?;
        let (input, _cfws) = optional(input, take_cfws);
        Ok((input, output))
    }

    pub fn take_mailbox(input: &[u8]) -> Res<Mailbox> {
        match_parsers(
            input,
            &mut [
                take_name_addr,
                (|input| take_addr_spec(input).map(|(i, m)| (i, (None, m))))
                    as fn(input: &[u8]) -> Res<Mailbox>,
            ][..],
        )
    }

    pub fn take_mailbox_list(input: &[u8]) -> Res<Vec<Mailbox>> {
        let mut mailboxes = Vec::new();
        let (mut input, first_mailbox) = take_mailbox(input)?;
        mailboxes.push(first_mailbox);

        while let Ok((new_input, new_mailbox)) = take_prefixed(input, take_mailbox, ",") {
            input = new_input;
            mailboxes.push(new_mailbox);
        }

        Ok((input, mailboxes))
    }

    pub fn take_group(input: &[u8]) -> Res<(Vec<String>, Vec<Mailbox>)> {
        let (input, display_name) = take_phrase(input)?;
        let (mut input, ()) = tag(input, b":")?;

        let group_list = if let Ok((new_input, mailbox_list)) = take_mailbox_list(input) {
            input = new_input;
            mailbox_list
        } else if let Ok((new_input, _cfws)) = take_cfws(input) {
            input = new_input;
            Vec::new()
        } else {
            Vec::new()
        };

        let (input, ()) = tag(input, b";")?;
        let (input, _cfws) = optional(input, take_cfws);
        Ok((input, (display_name, group_list)))
    }

    pub fn take_address(input: &[u8]) -> Res<Address> {
        if let Ok((input, mailbox)) = take_mailbox(input) {
            Ok((input, Address::Mailbox(mailbox)))
        } else if let Ok((input, group)) = take_group(input) {
            Ok((input, Address::Group(group)))
        } else {
            Err(Error::Known("Invalid address: not a mailbox nor a group"))
        }
    }

    pub fn take_address_list(input: &[u8]) -> Res<Vec<Address>> {
        let mut addresses = Vec::new();
        let (mut input, first_address) = take_address(input)?;
        addresses.push(first_address);

        while let Ok((new_input, new_address)) = take_prefixed(input, take_address, ",") {
            input = new_input;
            addresses.push(new_address);
        }

        Ok((input, addresses))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_local_part() {
            assert_eq!(take_local_part(b"mubelotix").unwrap().1, "mubelotix");
            assert_eq!(
                take_local_part(b"\"mubelotix\\ the\\ admin\"").unwrap().1,
                "mubelotix the admin"
            );
        }

        #[test]
        fn test_message_id() {
            assert_eq!(take_message_id(b"<idleft@idright>").unwrap().1.0, "idleft");
            assert_eq!(take_message_id(b"<idleft@idright>").unwrap().1.1, "idright");
            assert_eq!(take_message_id(b"<idleft@[idright]>").unwrap().1.1, "idright");
        }

        #[test]
        fn test_domain() {
            assert_eq!(
                take_domain_literal(b"[mubelotix.dev]").unwrap().1,
                "mubelotix.dev"
            );
            assert_eq!(
                take_domain_literal(b"[mubelotix\r\n .dev]").unwrap().1,
                "mubelotix.dev"
            );

            assert_eq!(
                take_domain(b"[mubelotix\r\n .dev]").unwrap().1,
                "mubelotix.dev"
            );
            assert_eq!(take_domain(b"mubelotix.dev").unwrap().1, "mubelotix.dev");
        }

        #[test]
        fn test_addr() {
            let (username, domain) = take_addr_spec(b"mubelotix@mubelotix.dev").unwrap().1;
            assert_eq!(username, "mubelotix");
            assert_eq!(domain, "mubelotix.dev");

            let (username, domain) = take_addr_spec(b"\"special\\ person\"@gmail.com").unwrap().1;
            assert_eq!(username, "special person");
            assert_eq!(domain, "gmail.com");

            let (name, (username, domain)) = take_name_addr(b"<mubelotix@gmail.com>").unwrap().1;
            assert!(name.is_none());
            assert_eq!(username, "mubelotix");
            assert_eq!(domain, "gmail.com");

            let (name, (username, domain)) =
                take_name_addr(b"Random Guy <someone@gmail.com>").unwrap().1;
            assert_eq!(name.unwrap().len(), 2);
            assert_eq!(username, "someone");
            assert_eq!(domain, "gmail.com");

            let (name, (username, domain)) = take_mailbox(b"mubelotix@mubelotix.dev").unwrap().1;
            assert!(name.is_none());
            assert_eq!(username, "mubelotix");
            assert_eq!(domain, "mubelotix.dev");

            let (name, (username, domain)) =
                take_mailbox(b"Random Guy <someone@gmail.com>").unwrap().1;
            assert_eq!(name.unwrap().len(), 2);
            assert_eq!(username, "someone");
            assert_eq!(domain, "gmail.com");
        }

        #[test]
        fn test_lists() {
            assert_eq!(
                take_mailbox_list(
                    b"test@gmail.com,Michel<michel@gmail.com>,<postmaster@mubelotix.dev>"
                )
                .unwrap()
                .1
                .len(),
                3
            );

            let (name, list) = take_group(
                b"Developers: Mubelotix <mubelotix@mubelotix.dev>, Someone <guy@gmail.com>;",
            )
            .unwrap()
            .1;
            assert_eq!(name[0], "Developers");
            assert_eq!(list[0].0.as_ref().unwrap()[0], "Mubelotix");
            assert_eq!(list[0].1.0, "mubelotix");
            assert_eq!(list[0].1.1, "mubelotix.dev");

            assert_eq!(take_address_list(b"mubelotix@gmail.com,guy@gmail.com,Developers:mubelotix@gmail.com,guy@gmail.com;").unwrap().1.len(), 3);
        }
    }
}

pub mod fields {
    use crate::prelude::*;
    use crate::{time::*, new_parser::address::*};
    use super::*;

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
        let (input, id) = address::take_message_id(input)?;
        let (input, ()) = tag(input, b"\r\n")?;

        Ok((input, id))
    }

    pub fn take_in_reply_to(input: &[u8]) -> Res<Vec<(String, String)>> {
        let (input, ()) = tag_no_case(input, b"In-Reply-To:", b"iN-rEPLY-tO:")?;
        let (input, ids) = take_many1(input, address::take_message_id)?;
        let (input, ()) = tag(input, b"\r\n")?;

        Ok((input, ids))
    }

    pub fn take_references(input: &[u8]) -> Res<Vec<(String, String)>> {
        let (input, ()) = tag_no_case(input, b"References:", b"rEFERENCES:")?;
        let (input, ids) = take_many1(input, address::take_message_id)?;
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
            assert_eq!(take_date(b"Date:5 May 2003 18:59:03 +0000\r\n").unwrap().1, (None, (5, Month::May, 2003), ((18, 59, 3), (true, 0, 0))));
        }

        #[test]
        fn test_originators() {
            assert_eq!(take_from(b"FrOm: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1[0].1.0, "mubelotix");
            assert_eq!(take_sender(b"sender: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1.1.1, "gmail.com");
            assert_eq!(take_reply_to(b"Reply-to: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1.len(), 1);
        }

        #[test]
        fn test_destination() {
            assert!(!take_to(b"To: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1.is_empty());
            assert!(!take_cc(b"Cc: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1.is_empty());
            assert!(!take_bcc(b"Bcc: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1.is_empty());
            assert!(take_bcc(b"Bcc: \r\n \r\n").unwrap().1.is_empty());
        }

        #[test]
        fn test_ids() {
            assert_eq!(take_message_id(b"Message-ID:<556100154@gmail.com>\r\n").unwrap().1.0, "556100154");
            assert_eq!(take_message_id(b"Message-ID:<556100154@gmail.com>\r\n").unwrap().1.1, "gmail.com");

            assert_eq!(take_references(b"References:<qzdzdq@qdz.com><dzdzjd@zdzdj.dz>\r\n").unwrap().1.len(), 2);
            
            assert_eq!(take_in_reply_to(b"In-Reply-To:<eefes@qzd.fr><52@s.dz><adzd@zd.d>\r\n").unwrap().1.len(), 3);
        }

        #[test]
        fn test_informational() {
            assert_eq!(take_subject(b"Subject:French school is boring\r\n").unwrap().1, "French school is boring");
            assert_eq!(take_subject(b"Subject:Folding\r\n is slow\r\n").unwrap().1, "Folding is slow");
            
            assert_eq!(take_comments(b"Comments:Rust is great\r\n").unwrap().1, "Rust is great");

            assert_eq!(take_keywords(b"Keywords:rust parser fast zero copy,email rfc5322\r\n").unwrap().1.len(), 2);
        }
    }
}

use crate::prelude::*;

pub fn take_atom(mut input: &[u8]) -> Result<(&[u8], String), Error> {
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    let (mut input, atom) =
        take_while1(input, is_atext).map_err(|_| Error::Known("Atom required"))?;
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    Ok((input, atom))
}

pub fn take_dot_atom_text(input: &[u8]) -> Result<(&[u8], String), Error> {
    let (mut input, mut output) = take_while1(input, is_atext)?;

    loop {
        if input.starts_with(b".") {
            if let Ok((new_input, atom)) = take_while1(&input[1..], is_atext) {
                output += String::Reference(&input[..1]);
                input = new_input;
                output += atom;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    Ok((input, output))
}

pub fn take_dot_atom(mut input: &[u8]) -> Result<(&[u8], String), Error> {
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    let (mut input, dot_atom) = take_dot_atom_text(input)?;
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    Ok((input, dot_atom))
}

pub fn take_word(input: &[u8]) -> Result<(&[u8], String), Error> {
    if let Ok((input, word)) = take_atom(input) {
        Ok((input, word))
    } else if let Ok((input, word)) = take_quoted_string(input) {
        Ok((input, word))
    } else {
        Err(Error::Known(
            "Word is not an atom and is not a quoted_string.",
        ))
    }
}

pub fn take_phrase(input: &[u8]) -> Result<(&[u8], Vec<String>), Error> {
    let mut words = Vec::new();
    let (mut input, word) = take_word(input)?;
    words.push(word);

    while let Ok((new_input, word)) = take_word(input) {
        input = new_input;
        words.push(word)
    }

    Ok((input, words))
}

pub fn take_unstructured(input: &[u8]) -> Result<(&[u8], String), Error> {
    let (mut input, output) = collect_many(input, |i| 
        collect_pair(i, 
            |i| Ok(take_fws(i).unwrap_or((i, String::Reference(&[])))), 
            |i| take_while1(i, is_vchar)
        )
    )?;

    while let Ok((new_input, _wsp)) = take_while1(input, is_wsp) {
        input = new_input;
    }

    Ok((input, output))
}

#[test]
fn test_word_and_phrase() {
    assert_eq!(take_word(b" this is a \"rust\\ test\" ").unwrap().1, "this");
    assert_eq!(
        take_phrase(b" this is a \"rust\\ test\" ").unwrap().1,
        vec!["this", "is", "a", "rust test"]
    );
}

#[test]
fn test_unstructured() {
    assert_eq!(
        take_unstructured(b"the quick brown fox jumps\r\n over the lazy dog   ")
            .unwrap()
            .1,
        "the quick brown fox jumps over the lazy dog"
    );
}

#[test]
fn test_quoted_pair() {
    assert!(take_quoted_pair(b"\\rtest").is_ok());
    assert!(take_quoted_pair(b"\\ test").is_ok());

    assert_eq!(take_quoted_pair(b"\\rtest").unwrap().1, "r");
    assert_eq!(take_quoted_pair(b"\\ test").unwrap().1, " ");

    assert!(take_quoted_pair(b"\\").is_err());
    assert!(take_quoted_pair(b"\\\0").is_err());
    assert!(take_quoted_pair(b"test").is_err());
}

#[test]
fn test_quoted_string() {
    assert_eq!(
        take_quoted_string(b" \"This\\ is\\ a\\ test\"").unwrap().1,
        "This is a test"
    );
    assert_eq!(
        take_quoted_string(b"\r\n  \"This\\ is\\ a\\ test\"  ")
            .unwrap()
            .1,
        "This is a test"
    );

    assert!(matches!(
        take_quoted_string(b"\r\n  \"This\\ is\\ a\\ test\"  ")
            .unwrap()
            .1,
        String::Owned(_)
    ));
    assert!(matches!(
        take_quoted_string(b"\r\n  \"hey\"  ").unwrap().1,
        String::Reference(_)
    ));
}

#[test]
fn test_atom() {
    assert_eq!(take_atom(b"this is a test").unwrap().1, "this");
    assert_eq!(take_atom(b"   averylongatom ").unwrap().1, "averylongatom");
    assert_eq!(
        take_dot_atom_text(b"this.is.a.test").unwrap().1,
        "this.is.a.test"
    );
    assert_eq!(
        take_dot_atom(b"  this.is.a.test ").unwrap().1,
        "this.is.a.test"
    );
}
