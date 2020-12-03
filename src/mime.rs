use crate::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug)]
pub struct RawEntity<'a> {
    pub mime_type: MimeType<'a>,
    pub subtype: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub id: Option<(Cow<'a, str>, Cow<'a, str>)>,
    pub parameters: HashMap<Cow<'a, str>, Cow<'a, str>>,
    pub value: Cow<'a, [u8]>,
}

impl<'a> RawEntity<'a> {
    pub fn parse(self) -> Result<Entity<'a>, Error> {
        crate::parsing::mime::entity::entity(self)
    }
}

#[derive(Debug)]
pub enum Entity<'a> {
    Multipart {
        subtype: Cow<'a, str>,
        content: Vec<RawEntity<'a>>,
    },
    Text {
        subtype: Cow<'a, str>,
        value: Cow<'a, str>,
    },
    Unknown(RawEntity<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum MimeType<'a> {
    Text,
    Image,
    Audio,
    Video,
    Application,
    Message,
    Multipart,
    Other(Cow<'a, str>),
}

impl<'a> MimeType<'a> {
    pub fn is_composite_type(&self) -> bool {
        match self {
            MimeType::Message => true,
            MimeType::Multipart => true,
            MimeType::Text => false,
            MimeType::Image => false,
            MimeType::Audio => false,
            MimeType::Video => false,
            MimeType::Application => false,
            MimeType::Other(_) => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ContentTransferEncoding<'a> {
    SevenBit,
    HeightBit,
    Binary,
    QuotedPrintable,
    Base64,
    Other(Cow<'a, str>),
}
