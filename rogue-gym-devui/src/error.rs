use error_chain_mini::{ChainedError, ErrorKind};
use rogue_gym_core::error::ErrorId as CoreError;
use std::io::Error as IoError;

#[derive(ErrorKind)]
pub enum ErrorID {
    #[msg(short = "core error", detailed = "{}", _0)]
    Core(CoreError),
    #[msg(short = "io error(from termion)", detailed = "{}", _0)]
    Io(IoError),
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

pub type Result<T> = ::std::result::Result<T, ChainedError<ErrorID>>;
