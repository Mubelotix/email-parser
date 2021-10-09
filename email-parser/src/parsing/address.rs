use crate::address::*;
use crate::prelude::*;
use std::borrow::Cow;

pub fn message_id(input: &[u8]) -> Res<(Cow<str>, Cow<str>)> {
    fn no_fold_litteral(input: &[u8]) -> Res<Cow<str>> {
        let (input, ()) = tag(
            input,
            b"[",
            "TAG ERROR: In message_id, a no_fold_litteral domain must be preceded by a `[`.",
        )?;
        let (input, domain) = take_while(input, is_dtext)?;
        let (input, ()) = tag(
            input,
            b"]",
            "TAG ERROR: In message_id, a no_fold_litteral domain must be closed by a `]`.",
        )?;
        Ok((input, Cow::Borrowed(domain)))
    }

    let (input, _cfws) = optional(input, cfws);
    let (input, ()) = tag(
        input,
        b"<",
        "TAG ERROR: A message ID must start with a `<`.",
    )?;
    let (input, id_left) = dot_atom_text(input)?;
    let (input, ()) = tag(
        input,
        b"@",
        "TAG ERROR: A message ID right part must be followed by a `@`.",
    )?;
    let (input, id_right) = match_parsers(input, &mut [dot_atom_text, no_fold_litteral][..])?;
    let (input, ()) = tag(
        input,
        b">",
        "TAG ERROR: A message ID left part must be followed by a `>`.",
    )?;
    let (input, _cfws) = optional(input, cfws);

    Ok((input, (id_left, id_right)))
}

pub fn addr_spec(input: &[u8]) -> Res<EmailAddress> {
    let (input, local_part) = local_part(input)?;
    let (input, ()) = tag(
        input,
        b"@",
        "TAG ERROR: An address local part must be followed by a `@`.",
    )?;
    let (input, domain) = domain(input)?;
    Ok((input, EmailAddress { local_part, domain }))
}

pub fn angle_addr(input: &[u8]) -> Res<EmailAddress> {
    let (input, _cfws) = optional(input, cfws);
    let (input, ()) = tag(
        input,
        b"<",
        "TAG ERROR: A angle_addr must start with a `<`.",
    )?;
    let (input, addr_spec) = addr_spec(input)?;
    let (input, ()) = tag(input, b">", "TAG ERROR: A angle_addr must end with a `>`.")?;
    let (input, _cfws) = optional(input, cfws);
    Ok((input, addr_spec))
}

pub fn name_addr(input: &[u8]) -> Res<Mailbox> {
    let (input, display_name) = if let (input, Some(display_name)) = optional(input, in_quotes) {
        (input, Some(display_name))
    } else {
        optional(input, phrase)
    };

    let (input, angle_addr) = angle_addr(input)?;

    Ok((
        input,
        Mailbox {
            name: display_name,
            address: angle_addr,
        },
    ))
}

pub fn local_part(input: &[u8]) -> Res<Cow<str>> {
    match_parsers(input, &mut [dot_atom, quoted_string][..])
}

pub fn domain(input: &[u8]) -> Res<Cow<str>> {
    match_parsers(input, &mut [dot_atom, domain_literal][..])
}

pub fn domain_literal<'a>(input: &'a [u8]) -> Res<Cow<'a, str>> {
    let (input, _cfws) = optional(input, cfws);
    let (mut input, ()) = tag(
        input,
        b"[",
        "TAG ERROR: A domain litteral must be preceded by a `[`.",
    )?;
    let mut output = empty_string();
    loop {
        let (new_input, _fws) = optional(input, fws);
        if let Ok((new_input, text)) = take_while1(new_input, is_dtext) {
            input = new_input;
            //add_string(&mut output, fws); should it be added?
            add_str(&mut output, text);
        } else {
            break;
        }
    }
    let (input, _fws) = optional(input, fws);
    let (input, ()) = tag(
        input,
        b"]",
        "TAG ERROR: A domain litteral must be followed by a `]`.",
    )?;
    let (input, _cfws) = optional(input, cfws);
    Ok((input, output))
}

pub fn mailbox(input: &[u8]) -> Res<Mailbox> {
    match_parsers(
        input,
        &mut [
            name_addr,
            (|input| {
                addr_spec(input).map(|(i, m)| {
                    (
                        i,
                        Mailbox {
                            name: None,
                            address: m,
                        },
                    )
                })
            }) as fn(input: &[u8]) -> Res<Mailbox>,
        ][..],
    )
}

pub fn mailbox_list(input: &[u8]) -> Res<Vec<Mailbox>> {
    let mut mailboxes = Vec::new();
    let (mut input, first_mailbox) = mailbox(input)?;
    mailboxes.push(first_mailbox);

    while let Ok((new_input, new_mailbox)) = prefixed(input, mailbox, ",") {
        input = new_input;
        mailboxes.push(new_mailbox);
    }

    let (input, _) = skip_whitespace(&input)?;

    Ok((input, mailboxes))
}

pub fn group(input: &[u8]) -> Res<(Vec<Cow<str>>, Vec<Mailbox>)> {
    let (input, display_name) = phrase(input)?;
    let (mut input, ()) = tag(
        input,
        b":",
        "TAG ERROR: A group display name must be followed by a `:`.",
    )?;

    let group_list = if let Ok((new_input, mailbox_list)) = mailbox_list(input) {
        input = new_input;
        mailbox_list
    } else if let Ok((new_input, _cfws)) = cfws(input) {
        input = new_input;
        Vec::new()
    } else {
        Vec::new()
    };

    let (input, ()) = tag(
        input,
        b";",
        "TAG ERROR: A group mailbox list must be closed by a `;`.",
    )?;
    let (input, _cfws) = optional(input, cfws);
    Ok((input, (display_name, group_list)))
}

pub fn address(input: &[u8]) -> Res<Address> {
    if let Ok((input, mailbox)) = mailbox(input) {
        Ok((input, Address::Mailbox(mailbox)))
    } else if let Ok((input, group)) = group(input) {
        Ok((input, Address::Group(group)))
    } else {
        Err(Error::Unknown("Invalid address: not a mailbox nor a group"))
    }
}

pub fn address_list(input: &[u8]) -> Res<Vec<Address>> {
    let mut addresses = Vec::new();
    let (mut input, first_address) = address(input)?;
    addresses.push(first_address);

    while let Ok((new_input, new_address)) = prefixed(input, address, ",") {
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
        assert_eq!(local_part(b"mubelotix").unwrap().1, "mubelotix");
        assert_eq!(
            local_part(b"\"mubelotix\\ the\\ admin\"").unwrap().1,
            "mubelotix the admin"
        );
    }

    #[test]
    fn test_message_id() {
        assert_eq!(message_id(b"<idleft@idright>").unwrap().1 .0, "idleft");
        assert_eq!(message_id(b"<idleft@idright>").unwrap().1 .1, "idright");
        assert_eq!(message_id(b"<idleft@[idright]>").unwrap().1 .1, "idright");
    }

    #[test]
    fn test_domain() {
        assert_eq!(
            domain_literal(b"[mubelotix.dev]").unwrap().1,
            "mubelotix.dev"
        );
        assert_eq!(
            domain_literal(b"[mubelotix\r\n .dev]").unwrap().1,
            "mubelotix.dev"
        );

        assert_eq!(domain(b"[mubelotix\r\n .dev]").unwrap().1, "mubelotix.dev");
        assert_eq!(domain(b"mubelotix.dev").unwrap().1, "mubelotix.dev");
    }

    #[test]
    fn test_addr() {
        let address = addr_spec(b"mubelotix@mubelotix.dev").unwrap().1;
        assert_eq!(address.local_part, "mubelotix");
        assert_eq!(address.domain, "mubelotix.dev");

        let address = addr_spec(b"\"special\\ person\"@gmail.com").unwrap().1;
        assert_eq!(address.local_part, "special person");
        assert_eq!(address.domain, "gmail.com");

        let mlbx = name_addr(b"<mubelotix@gmail.com>").unwrap().1;
        assert!(mlbx.name.is_none());
        assert_eq!(mlbx.address.local_part, "mubelotix");
        assert_eq!(mlbx.address.domain, "gmail.com");

        let mlbx = name_addr(b"Random Guy <someone@gmail.com>").unwrap().1;
        assert_eq!(mlbx.name.unwrap().len(), 2);
        assert_eq!(mlbx.address.local_part, "someone");
        assert_eq!(mlbx.address.domain, "gmail.com");

        let mlbx = mailbox(b"mubelotix@mubelotix.dev").unwrap().1;
        assert!(mlbx.name.is_none());
        assert_eq!(mlbx.address.local_part, "mubelotix");
        assert_eq!(mlbx.address.domain, "mubelotix.dev");

        let mlbx = mailbox(b"Random Guy <someone@gmail.com>").unwrap().1;
        assert_eq!(mlbx.name.unwrap(), vec!["Random", "Guy"]);
        assert_eq!(mlbx.address.local_part, "someone");
        assert_eq!(mlbx.address.domain, "gmail.com");
    }

    #[test]
    fn test_lists() {
        assert_eq!(
            mailbox_list(b"test@gmail.com,Michel<michel@gmail.com>,<postmaster@mubelotix.dev>")
                .unwrap()
                .1
                .len(),
            3
        );

        let (name, list) =
            group(b"Developers: Mubelotix <mubelotix@mubelotix.dev>, Someone <guy@gmail.com>;")
                .unwrap()
                .1;
        assert_eq!(name[0], "Developers");
        assert_eq!(list[0].name.as_ref().unwrap(), &vec!["Mubelotix"]);
        assert_eq!(list[0].address.local_part, "mubelotix");
        assert_eq!(list[0].address.domain, "mubelotix.dev");

        assert_eq!(
            address_list(
                b"mubelotix@gmail.com,guy@gmail.com,Developers:mubelotix@gmail.com,guy@gmail.com;"
            )
            .unwrap()
            .1
            .len(),
            3
        );
    }
}
