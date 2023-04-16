use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, Error>;

pub type Error = ErrorKind;

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Invalid input parameter:{input}")]
    InputError{
      input: String,
    },

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ProtoError(#[from] solana_gossip_proto::errors::Error),
}