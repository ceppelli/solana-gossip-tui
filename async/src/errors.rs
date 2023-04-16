use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, Error>;

pub type Error = ErrorKind;

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Invalid input parameter")]
    InputError,

    #[error("Timeout error")]
    TimeouttError,

    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ProtoError(#[from] solana_gossip_proto::errors::Error),
}
