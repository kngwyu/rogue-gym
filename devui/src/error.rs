//! our error types
use log::SetLoggerError;
pub use rogue_gym_core::error::{FailExt, GameResult, ResultExt1, ResultExt2};
use std::io::Error as IoError;

#[derive(Debug, Fail)]
pub enum ErrorID {
    #[fail(display = "io error: {:?}", _0)]
    Io(IoError),
    #[fail(display = "Invalid Command Args")]
    InvalidArg,
    #[fail(display = "Invalid screen size width: {} height:{} ", _0, _1)]
    InvalidScreenSize(u16, u16),
    #[fail(display = "Error in logging: {:?}", _0)]
    Log(SetLoggerError),
}

impl FailExt for ErrorID {}

impl From<IoError> for ErrorID {
    fn from(e: IoError) -> ErrorID {
        ErrorID::Io(e)
    }
}

impl From<SetLoggerError> for ErrorID {
    fn from(e: SetLoggerError) -> ErrorID {
        ErrorID::Log(e)
    }
}
