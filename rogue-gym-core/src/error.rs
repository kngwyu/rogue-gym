use dungeon::{X, Y};
use rect_iter::IndexError;
// crate local re-exports

/// Our own ErrorKind type
#[derive(Clone, Debug, ErrorKind)]
pub enum ErrorId {
    #[msg(short = "Invalid index access", detailed = "x: {:?}, y: {:?}", x, y)]
    Index { x: Option<X>, y: Option<Y> },
}

impl From<IndexError> for ErrorId {
    fn from(e: IndexError) -> Self {
        ErrorId::Index {
            x: Some(X(e.x as i32)),
            y: Some(Y(e.y as i32)),
        }
    }
}
