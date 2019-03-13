use crate::input::{InputCode, Key};
use failure::{self, Error, Fail};
use rect_iter::IndexError;
use serde_json::Error as JsonError;
use std::error::Error as StdError;
use std::fmt;

pub type GameResult<T> = Result<T, Error>;

pub trait ResultExt1<T, E> {
    fn chain_err<F, D>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> D,
        D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> ResultExt1<T, E> for Result<T, E>
where
    E: Into<Error>,
{
    fn chain_err<F, D>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> D,
        D: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| err.into().context(f()).into())
    }
}

pub trait ResultExt2<T, E> {
    fn into_chained<F, D>(self, f: F) -> GameResult<T>
    where
        F: FnOnce() -> D,
        D: fmt::Display + Send + Sync + 'static;

    fn compat(self) -> GameResult<T>;
}

impl<T, E> ResultExt2<T, E> for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn into_chained<F, D>(self, f: F) -> GameResult<T>
    where
        F: FnOnce() -> D,
        D: fmt::Display + Send + Sync + 'static,
    {
        failure::ResultExt::compat(self).map_err(|err| err.context(f()).into())
    }

    fn compat(self) -> GameResult<T> {
        failure::ResultExt::compat(self).map_err(|e| e.into())
    }
}

pub trait FailExt: Fail + Sized {
    fn into_with<F, D>(self, f: F) -> failure::Error
    where
        F: FnOnce() -> D,
        D: fmt::Display + Send + Sync + 'static,
    {
        self.context(f()).into()
    }
}

/// Our own ErrorKind type
#[derive(Debug, Fail)]
pub enum ErrorId {
    #[fail(display = "Invalid index access: {:?}", _0)]
    Index(IndexError),
    #[fail(display = "Invliad input key: {:?}", _0)]
    InvalidInput(Key),
    #[fail(display = "Ignored input code: {:?}", _0)]
    IgnoredInput(InputCode),
    #[fail(display = "Incomplete input")]
    IncompleteInput,
    #[fail(display = "Invalid value in setting")]
    InvalidSetting,
    #[fail(display = "Json parsing error: {}", _0)]
    Json(JsonError),
    #[fail(display = "Invalid conversion")]
    InvalidConversion,
    #[fail(display = "Maybe software bug")]
    MaybeBug,
    // STUB,
    #[fail(display = "Unimplemented")]
    Unimplemented,
}

impl FailExt for ErrorId {}

impl ErrorId {
    pub fn can_allow(&self) -> bool {
        use self::ErrorId::*;
        match self {
            InvalidInput(_) | IgnoredInput(_) | IncompleteInput => true,
            _ => false,
        }
    }
}

impl From<IndexError> for ErrorId {
    fn from(e: IndexError) -> Self {
        ErrorId::Index(e)
    }
}

impl From<JsonError> for ErrorId {
    fn from(e: JsonError) -> Self {
        ErrorId::Json(e)
    }
}
