use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Mailbox<'a> {
    /// The name associated with an email.\
    /// Each name is stored individually in the `Vec`. For example "Elton John" results in `Some(["Elton", "John"])`.
    pub name: Option<Vec<Cow<'a, str>>>,
    pub address: EmailAddress<'a>,
}

#[derive(Debug, Clone)]
pub struct EmailAddress<'a> {
    pub local_part: Cow<'a, str>,
    pub domain: Cow<'a, str>,
}

#[derive(Debug, Clone)]
pub enum Address<'a> {
    Mailbox(Mailbox<'a>),
    Group((Vec<Cow<'a, str>>, Vec<Mailbox<'a>>)),
}
