#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Explicit(&'static str),
    Unknown(&'static str),
    DuplicateHeader(&'static str),
    MissingHeader(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unknown(message) => write!(f, "{}", message),
            Error::Explicit(message) => write!(f, "{}", message),
            Error::DuplicateHeader(name) => {
                write!(f, "There are too many {} headers in this mail.", name)
            }
            Error::MissingHeader(name) => write!(f, "A valid {} header is required.", name),
        }
    }
}

impl std::error::Error for Error {}

pub type Res<'a, T> = Result<(&'a [u8], T), Error>;
