use error_chain_mini::ChainedError;
pub use error_chain_mini::{ErrorKind, ResultExt};
use input::{InputCode, Key};
use rect_iter::IndexError;
use serde_json::Error as JsonError;

/// Our own ErrorKind type
#[derive(ErrorKind)]
pub enum ErrorId {
    #[msg(short = "Invalid index access", detailed = "{:?}", _0)]
    Index(IndexError),
    #[msg(
        short = "invliad input which is not mapped by keymap",
        detailed = "key: {:?}",
        _0
    )]
    InvalidInput(Key),
    #[msg(short = "ignored input", detailed = "code: {:?}", _0)]
    IgnoredInput(InputCode),
    IncompleteInput,
    #[msg(short = "Invalid value in setting")]
    InvalidSetting,
    #[msg(short = "json", detailed = "{}", _0)]
    Json(JsonError),
    #[msg(short = "invalid conversion")]
    InvalidConversion,
    MaybeBug,
    // STUB
    Unimplemented,
}

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

pub type GameError = ChainedError<ErrorId>;

pub type GameResult<T> = Result<T, GameError>;
