use dungeon::{X, Y};
use error_chain_mini::ChainedError;
use input::Key;
use rect_iter::IndexError;
/// Our own ErrorKind type
#[derive(Clone, Debug, ErrorKind)]
pub enum ErrorId {
    #[msg(short = "Invalid index access", detailed = "x: {:?}, y: {:?}", x, y)]
    Index { x: Option<X>, y: Option<Y> },
    #[msg(short = "Invalid Input", detailed = "key: {:?}", _0)]
    Input(Key),
}

impl From<IndexError> for ErrorId {
    fn from(e: IndexError) -> Self {
        ErrorId::Index {
            x: Some(X(e.x as i32)),
            y: Some(Y(e.y as i32)),
        }
    }
}

pub type GameError = ChainedError<ErrorId>;

pub type GameResult<T> = Result<T, GameError>;
