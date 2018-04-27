use dungeon::{X, Y};
use error_chain_mini::ChainedError;
pub use error_chain_mini::ResultExt;
use input::Key;
use rect_iter::IndexError;
/// Our own ErrorKind type
#[derive(Clone, Debug, ErrorKind)]
pub enum ErrorId {
    #[msg(short = "Invalid index access", detailed = "{:?}", _0)]
    Index(IndexError),
    #[msg(short = "Invalid Input", detailed = "key: {:?}", _0)]
    Input(Key),
    #[msg(short = "Incomplete Input")]
    IncompleteInput,
}

impl From<IndexError> for ErrorId {
    fn from(e: IndexError) -> Self {
        ErrorId::Index(e)
    }
}

pub type GameError = ChainedError<ErrorId>;

pub type GameResult<T> = Result<T, GameError>;
