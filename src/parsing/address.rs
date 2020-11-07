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
    let (input, id_right) =
        match_parsers(input, &mut [take_dot_atom_text, take_no_fold_litteral][..])?;
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

pub fn take_angle_addr(input: &[u8]) -> Res<(String, String)> {
    let (input, _cfws) = optional(input, take_cfws);
    let (input, ()) = tag(input, b"<")?;
    let (input, addr_spec) = take_addr_spec(input)?;
    let (input, ()) = tag(input, b">")?;
    let (input, _cfws) = optional(input, take_cfws);
    Ok((input, addr_spec))
}

pub fn take_name_addr(input: &[u8]) -> Res<Mailbox> {
    let (input, display_name) = optional(input, take_phrase);
    let (input, angle_addr) = take_angle_addr(input)?;

    Ok((input, (display_name, angle_addr)))
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

        let (name, (username, domain)) = take_mailbox(b"Random Guy <someone@gmail.com>").unwrap().1;
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

        assert_eq!(
            take_address_list(
                b"mubelotix@gmail.com,guy@gmail.com,Developers:mubelotix@gmail.com,guy@gmail.com;"
            )
            .unwrap()
            .1
            .len(),
            3
        );
    }
}
