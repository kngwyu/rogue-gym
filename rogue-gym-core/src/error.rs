use dungeon::{X, Y};
use error_chain_mini::ChainedError;
pub(crate) use error_chain_mini::ErrorKind;
pub(crate) use error_chain_mini::ResultExt;
use input::{InputCode, Key};
use rect_iter::IndexError;
/// Our own ErrorKind type
#[derive(Clone, Debug, ErrorKind)]
pub enum ErrorId {
    #[msg(short = "Invalid index access", detailed = "{:?}", _0)]
    Index(IndexError),
    #[msg(short = "invliad input which is not mapped by keymap", detailed = "key: {:?}", _0)]
    InvalidInput(Key),
    #[msg(short = "ignored input", detailed = "code: {:?}", _0)]
    IgnoredInput(InputCode),
    #[msg(short = "Incomplete input")]
    IncompleteInput,
    #[msg(short = "Invalid Setting")]
    InvalidSetting,
    Unimplemented,
    // it's intended to use only in 'immediate panic pattern'
    LogicError,
}

impl From<IndexError> for ErrorId {
    fn from(e: IndexError) -> Self {
        ErrorId::Index(e)
    }
}

pub type GameError = ChainedError<ErrorId>;

pub type GameResult<T> = Result<T, GameError>;
