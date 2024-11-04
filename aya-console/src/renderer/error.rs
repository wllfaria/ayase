pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Memory,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<aya_cpu::memory::Error> for Error {
    fn from(_: aya_cpu::memory::Error) -> Self {
        Self::Memory
    }
}
