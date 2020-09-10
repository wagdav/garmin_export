pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidInput(String),
    IOError(String),
    APIError(String),
    Forbidden,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        error
            .status()
            .map_or(Error::APIError(error.to_string()), |status| {
                if status == reqwest::StatusCode::FORBIDDEN {
                    Error::Forbidden
                } else {
                    Error::APIError(error.to_string())
                }
            })
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error.to_string())
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(error: zip::result::ZipError) -> Self {
        Error::IOError(error.to_string())
    }
}
