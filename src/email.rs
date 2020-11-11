use crate::prelude::*;
use std::borrow::Cow;

pub struct Email<'a> {
    pub body: Option<Cow<'a, str>>,
    pub fields: Vec<Field<'a>>,
}

impl<'a> std::convert::TryFrom<&'a [u8]> for Email<'a> {
    type Error = crate::error::Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let (fields, body) = crate::parse_message(value)?;

        Ok(Email { body, fields })
    }
}
