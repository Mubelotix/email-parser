use std::borrow::Cow;

#[derive(Debug)]
pub enum Entity<'a> {
    Multipart(Vec<Entity<'a>>),
    Text {subtype: Cow<'a, str>, value: Cow<'a, str>},
    Unknown{mime_type: MimeType<'a>, subtype: Cow<'a, str>, value: Cow<'a, [u8]>}
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
