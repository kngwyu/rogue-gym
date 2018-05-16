//! our error types
use error_chain_mini::{ChainedError, ErrorKind};
use rogue_gym_core::dungeon::Coord;
use rogue_gym_core::error::ErrorId as CoreError;
use std::io::Error as IoError;
pub(crate) type Result<T> = ::std::result::Result<T, ChainedError<ErrorID>>;

#[derive(ErrorKind)]
pub(crate) enum ErrorID {
    #[msg(short = "core error", detailed = "{}", _0)]
    Core(CoreError),
    #[msg(short = "io error", detailed = "{}", _0)]
    Io(IoError),
    #[msg(short = "Invalid Command Args")]
    InvalidArg,
    #[msg(short = "Invalid screen size", detailed = "width: {} height: {}", _0, _1)]
    InvalidScreenSize(u16, u16),
}

impl From<CoreError> for ErrorID {
    fn from(e: CoreError) -> ErrorID {
        ErrorID::Core(e)
    }
}

impl From<IoError> for ErrorID {
    fn from(e: IoError) -> ErrorID {
        ErrorID::Io(e)
    }
}
