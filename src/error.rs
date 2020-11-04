#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Known(&'static str),
}

pub type Res<'a, T> = Result<(&'a [u8], T), Error>;
