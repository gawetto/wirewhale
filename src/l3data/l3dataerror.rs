use crate::read::UntilReadError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum L3dataError {
    #[error(transparent)]
    Io(#[from] UntilReadError),
}
