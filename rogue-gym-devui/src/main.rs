extern crate error_chain_mini;
extern crate rogue_gym_core;
extern crate termion;
#[macro_use]
extern crate error_chain_mini_derive;

mod error;
use error::{ErrorID, Result};
use error_chain_mini::{ErrorKind, ResultExt};
use rogue_gym_core::{GameConfig, RunTime};
use termion::event::{Event, Key};

fn main() {}

fn play_game() {}
