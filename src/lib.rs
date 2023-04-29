use std::{cell::UnsafeCell, mem::MaybeUninit, pin::Pin, borrow::Cow};

use faster_pest::*;

#[derive(Parser)]
#[grammar = "src/main.pest"]
pub struct Parser {

}

#[derive(Debug)]
pub enum EmailValidationError {
    ParsingError(Error),
    MissingHeader(&'static str),
    TooManyHeaders(&'static str),
}

impl From<Error> for EmailValidationError {
    fn from(e: Error) -> Self {
        EmailValidationError::ParsingError(e)
    }
}

pub struct Email<'i> {
    idents: IdentList<Ident<'i>>,
    origination_date_idx: usize,
    from_idx: usize,
    sender_idx: Option<usize>,
    reply_to_idx: Option<usize>,
    to_idx: Option<usize>,
    cc_idx: Option<usize>,
    bcc_idx: Option<usize>,
    message_id_idx: Option<usize>,
    in_reply_to_idx: Option<usize>,
    references_idx: Option<usize>,
    subject_idx: Option<usize>,
}

impl<'i> Email<'i> {
    pub fn parse(raw: &'i str) -> Result<Self, EmailValidationError> {
        let idents = Parser::parse_message(raw)?;

        let (mut origination_date_idx, mut from_idx, mut sender_idx, mut reply_to_idx, mut to_idx, mut cc_idx, mut bcc_idx, mut message_id_idx, mut in_reply_to_idx, mut references_idx, mut subject_idx) = (None, None, None, None, None, None, None, None, None, None, None);

        for child in idents.root().children() {
            match child.as_rule() {
                Rule::origination_date => {
                    if origination_date_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("Date"));
                    }
                    origination_date_idx = Some(child.idx());
                },
                Rule::from => {
                    if from_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("From"));
                    }
                    from_idx = Some(child.idx());
                },
                Rule::sender => {
                    if sender_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("Sender"));
                    }
                    sender_idx = Some(child.idx());
                },
                Rule::reply_to => {
                    if reply_to_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("Reply-To"));
                    }
                    reply_to_idx = Some(child.idx());
                },
                Rule::to => {
                    if to_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("To"));
                    }
                    to_idx = Some(child.idx());
                },
                Rule::cc => {
                    if cc_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("Cc"));
                    }
                    cc_idx = Some(child.idx());
                },
                Rule::bcc => {
                    if bcc_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("Bcc"));
                    }
                    bcc_idx = Some(child.idx());
                },
                Rule::message_id => {
                    if message_id_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("Message-ID"));
                    }
                    message_id_idx = Some(child.idx());
                },
                Rule::in_reply_to => {
                    if in_reply_to_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("In-Reply-To"));
                    }
                    in_reply_to_idx = Some(child.idx());
                },
                Rule::references => {
                    if references_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("References"));
                    }
                    references_idx = Some(child.idx());
                },
                Rule::subject => {
                    if subject_idx.is_some() {
                        return Err(EmailValidationError::TooManyHeaders("Subject"));
                    }
                    subject_idx = Some(child.idx());
                },
                _ => (),
            }
        }
        
        Ok(Email {
            idents,
            origination_date_idx: origination_date_idx.ok_or(EmailValidationError::MissingHeader("Date"))?,
            from_idx: from_idx.ok_or(EmailValidationError::MissingHeader("From"))?,
            sender_idx,
            reply_to_idx,
            to_idx,
            cc_idx,
            bcc_idx,
            message_id_idx,
            in_reply_to_idx,
            references_idx,
            subject_idx
        })
    }

    pub fn origination_date_ident(&self) -> IdentRef<Ident<'i>> {
        unsafe { self.idents.get_unchecked(self.origination_date_idx) }
    }

    pub fn from_ident(&self) -> IdentRef<Ident<'i>> {
        unsafe { self.idents.get_unchecked(self.from_idx) }
    }

    pub fn sender_ident(&self) -> Option<IdentRef<Ident<'i>>> {
        self.sender_idx.map(|idx| unsafe { self.idents.get_unchecked(idx) })
    }

    pub fn reply_to_ident(&self) -> Option<IdentRef<Ident<'i>>> {
        self.reply_to_idx.map(|idx| unsafe { self.idents.get_unchecked(idx) })
    }

    pub fn to_ident(&self) -> Option<IdentRef<Ident<'i>>> {
        self.to_idx.map(|idx| unsafe { self.idents.get_unchecked(idx) })
    }

    pub fn cc_ident(&self) -> Option<IdentRef<Ident<'i>>> {
        self.cc_idx.map(|idx| unsafe { self.idents.get_unchecked(idx) })
    }

    pub fn bcc_ident(&self) -> Option<IdentRef<Ident<'i>>> {
        self.bcc_idx.map(|idx| unsafe { self.idents.get_unchecked(idx) })
    }

    pub fn message_id_ident(&self) -> Option<IdentRef<Ident<'i>>> {
        self.message_id_idx.map(|idx| unsafe { self.idents.get_unchecked(idx) })
    }

    pub fn in_reply_to_ident(&self) -> Option<IdentRef<Ident<'i>>> {
        self.in_reply_to_idx.map(|idx| unsafe { self.idents.get_unchecked(idx) })
    }

    pub fn references_ident(&self) -> Option<IdentRef<Ident<'i>>> {
        self.references_idx.map(|idx| unsafe { self.idents.get_unchecked(idx) })
    }

    pub fn subject_ident(&self) -> Option<IdentRef<Ident<'i>>> {
        self.subject_idx.map(|idx| unsafe { self.idents.get_unchecked(idx) })
    }
    pub fn raw_subject(&self) -> Option<&str> {
        self.subject_ident().map(|ident| ident.as_str())
    }
    pub fn subject(&self) -> Option<Cow<str>> {
        self.subject_ident().map(move |i| i.children().join_all())
    }
}
