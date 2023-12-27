pub trait ErrorT: std::error::Error + Send + Sync + 'static {}

impl<E: std::error::Error + Send + Sync + 'static> ErrorT for E {}

impl<E: ErrorT> From<E> for Box<dyn ErrorT> {
    fn from(err: E) -> Self {
        Box::new(err)
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GroupChatTaskError {
    #[error("group chat task error")]
    SendError,

    #[error("group chat task error")]
    OtherError(Box<dyn ErrorT>),
}
