pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    NonUtf8(&'static str),
    SyntaxError,
    Miette(miette::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<miette::Error> for Error {
    fn from(err: miette::Error) -> Self {
        Self::Miette(err)
    }
}
