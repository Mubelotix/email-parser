use crate::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

/// A generic MIME Entity.
#[derive(Debug, PartialEq, Clone)]
pub struct RawEntity<'a> {
    pub mime_type: ContentType<'a>,
    /// The subtype (in lowercase).
    pub subtype: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub id: Option<(Cow<'a, str>, Cow<'a, str>)>,
    /// Parameters named in lowercase.
    pub parameters: HashMap<Cow<'a, str>, Cow<'a, str>>,
    #[cfg(feature = "content-disposition")]
    pub disposition: Option<Disposition<'a>>,
    /// The raw value of this entity.
    /// It has already been decoded.
    pub value: Cow<'a, [u8]>,
    pub additional_headers: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> RawEntity<'a> {
    /// Use this function to decode [text](Entity::Text) and [multipart](Entity::Multipart) values.\
    /// If this library is not able to provide a higher-level structure, the data will be returned [untouched]([Entity::Unknown]).\
    /// If this entity is supported but is wrongly formatted, an error will be returned.
    pub fn parse(&'a self) -> Result<Entity<'a>, Error> {
        crate::parsing::mime::entity::entity(self)
    }
}

/// A higher-level reprentation of entities.\
/// Can be obtained with [RawEntity::parse].
#[derive(Debug, PartialEq, Clone)]
pub enum Entity<'a> {
    /// A multipart entity is an array of entities.\
    /// See the subtype for information about their relation.
    Multipart {
        subtype: &'a Cow<'a, str>,
        content: Vec<RawEntity<'a>>,
    },
    /// A decoded text entity.\
    /// Supported charsets are all ISO, US-ASCII and UTF-8.
    Text {
        subtype: &'a Cow<'a, str>,
        value: Cow<'a, str>,
    },
    /// All other entities that are not supported by this library.
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ContentType<'a> {
    Text,
    Image,
    Audio,
    Video,
    Application,
    Message,
    Multipart,
    Unknown(Cow<'a, str>),
}

impl<'a> ContentType<'a> {
    /// Extends the lifetime from `'a` to `'static` by guaranteeing that we have ownership after calling this function.
    /// It will call `to_owned` on references.\
    /// Since there are rarely references, this is almost always free.
    pub fn into_owned(self) -> ContentType<'static> {
        match self {
            ContentType::Text => ContentType::Text,
            ContentType::Image => ContentType::Image,
            ContentType::Audio => ContentType::Audio,
            ContentType::Video => ContentType::Video,
            ContentType::Application => ContentType::Application,
            ContentType::Message => ContentType::Message,
            ContentType::Multipart => ContentType::Multipart,
            ContentType::Unknown(Cow::Owned(value)) => ContentType::Unknown(Cow::Owned(value)),
            ContentType::Unknown(Cow::Borrowed(value)) => {
                ContentType::Unknown(Cow::Owned(value.to_owned()))
            }
        }
    }
}

/// Information about how a [RawEntity] must be displayed.\
/// Is accessible from [Disposition::disposition_type].
#[derive(Debug, PartialEq, Clone)]
pub enum DispositionType<'a> {
    /// An inline entity\
    /// [Learn more](https://tools.ietf.org/html/rfc2183#section-2.1)
    Inline,
    /// An attachment\
    /// [Learn more](https://tools.ietf.org/html/rfc2183#section-2.2)
    Attachment,
    /// An unknown content-disposition. Should be treated as [DispositionType::Attachment].\
    /// [Learn more](https://tools.ietf.org/html/rfc2183#section-2.8).
    Unknown(Cow<'a, str>),
}

impl<'a> DispositionType<'a> {
    /// Extends the lifetime from `'a` to `'static` by guaranteeing that we have ownership after calling this function.
    /// It will call `to_owned` on references.\
    /// Since there are rarely references, this is almost always free.
    pub fn into_owned(self) -> DispositionType<'static> {
        match self {
            DispositionType::Inline => DispositionType::Inline,
            DispositionType::Attachment => DispositionType::Attachment,
            DispositionType::Unknown(Cow::Owned(value)) => {
                DispositionType::Unknown(Cow::Owned(value))
            }
            DispositionType::Unknown(Cow::Borrowed(value)) => {
                DispositionType::Unknown(Cow::Owned(value.to_owned()))
            }
        }
    }
}

/// Some information about how to display a [RawEntity] and some file metadata.\
/// Is accessible from [RawEntity::disposition].\
/// The size parameter is not directly supported as it is the "approximate size". You can get the exact size in bytes by calling `.len()` on the value of an [RawEntity::value].
#[derive(Debug, PartialEq, Clone)]
pub struct Disposition<'a> {
    pub disposition_type: DispositionType<'a>,
    pub filename: Option<Cow<'a, str>>,
    pub creation_date: Option<DateTime>,
    pub modification_date: Option<DateTime>,
    pub read_date: Option<DateTime>,
    pub unstructured: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> Disposition<'a> {
    /// Extends the lifetime from `'a` to `'static` by guaranteeing that we have ownership after calling this function.
    /// It will call `to_owned` on references.
    pub fn into_owned(self) -> Disposition<'static> {
        Disposition {
            disposition_type: self.disposition_type.into_owned(),
            filename: self
                .filename
                .map(|filename| Cow::Owned(filename.into_owned())),
            creation_date: self.creation_date,
            modification_date: self.modification_date,
            read_date: self.read_date,
            unstructured: self
                .unstructured
                .into_iter()
                .map(|(n, v)| (Cow::Owned(n.into_owned()), Cow::Owned(v.into_owned())))
                .collect(),
        }
    }
}

impl<'a> ContentType<'a> {
    pub fn is_composite_type(&self) -> bool {
        match self {
            ContentType::Message => true,
            ContentType::Multipart => true,
            ContentType::Text => false,
            ContentType::Image => false,
            ContentType::Audio => false,
            ContentType::Video => false,
            ContentType::Application => false,
            ContentType::Unknown(_) => false,
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
    Unknown(Cow<'a, str>),
}

impl<'a> ContentTransferEncoding<'a> {
    /// Extends the lifetime from `'a` to `'static` by guaranteeing that we have ownership after calling this function.
    /// It will call `to_owned` on references.\
    /// Since there are rarely references, this is almost always free.
    pub fn into_owned(self) -> ContentTransferEncoding<'static> {
        match self {
            ContentTransferEncoding::SevenBit => ContentTransferEncoding::SevenBit,
            ContentTransferEncoding::HeightBit => ContentTransferEncoding::HeightBit,
            ContentTransferEncoding::Binary => ContentTransferEncoding::Binary,
            ContentTransferEncoding::QuotedPrintable => ContentTransferEncoding::QuotedPrintable,
            ContentTransferEncoding::Base64 => ContentTransferEncoding::Base64,
            ContentTransferEncoding::Unknown(Cow::Owned(value)) => {
                ContentTransferEncoding::Unknown(Cow::Owned(value))
            }
            ContentTransferEncoding::Unknown(Cow::Borrowed(value)) => {
                ContentTransferEncoding::Unknown(Cow::Owned(value.to_owned()))
            }
        }
    }
}
