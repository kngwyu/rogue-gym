use crate::input::{InputCode, Key};
use rect_iter::IndexError;
use serde_json::Error as JsonError;
use std::borrow::Cow;
use thiserror::Error;

pub type GameResult<T> = Result<T, anyhow::Error>;

/// Our own ErrorKind type
#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Invalid index access: {:?}", _0)]
    Index(IndexError),
    #[error("Invliad input key: {:?}", _0)]
    InvalidInput(Key),
    #[error("Ignored input code: {:?}", _0)]
    IgnoredInput(InputCode),
    #[error("Incomplete input")]
    IncompleteInput,
    #[error("Invalid Setting: {}", _0)]
    InvalidSetting(Cow<'static, str>),
    #[error("Json parsing error: {}", _0)]
    Json(JsonError),
    #[error("Invalid conversion")]
    InvalidConversion,
    #[error("Maybe bug: {}", _0)]
    MaybeBug(&'static str),
    // STUB,
    #[error("{} is unimplemented", _0)]
    Unimplemented(&'static str),
}

impl ErrorKind {
    pub fn can_allow(&self) -> bool {
        use self::ErrorKind::*;
        match self {
            InvalidInput(_) | IgnoredInput(_) | IncompleteInput => true,
            _ => false,
        }
    }
}

impl From<IndexError> for ErrorKind {
    fn from(e: IndexError) -> Self {
        ErrorKind::Index(e)
    }
}

impl From<JsonError> for ErrorKind {
    fn from(e: JsonError) -> Self {
        ErrorKind::Json(e)
    }
}
