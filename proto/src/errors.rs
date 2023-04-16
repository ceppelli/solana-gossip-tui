use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, Error>;

//pub type Error = Box<ErrorKind>;
pub type Error = ErrorKind;

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Decode error")]
    DecodeError,

    #[error("Encode error")]
    EncodeError,

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    BincodeError(#[from] bincode::Error),
}
