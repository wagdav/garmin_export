pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidInput(String),
    RequestFailed(String),
    UnexpectedServerResponse,
}