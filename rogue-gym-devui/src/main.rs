extern crate clap;
extern crate error_chain_mini;
extern crate rogue_gym_core;
extern crate termion;
#[macro_use]
extern crate error_chain_mini_derive;

mod error;
use clap::ArgMatches;
use error::{ErrorID, Result};
use error_chain_mini::{ErrorKind, ResultExt};
use rogue_gym_core::{GameConfig, RunTime};
use std::fs::File;
use std::io::Read;
use termion::event::{Event, Key};

fn main() {
    if let Err(err) = play_game() {
        println!("Error! {}", err);
        std::process::exit(1);
    }
}

fn parse_args<'a>() -> ArgMatches<'a> {
    clap::App::new("rogue-gym developper ui")
        .version("0.0.1")
        .author("Yuji Kanagawa <yuji.kngw.80s.revive@gmail.com>")
        .about("play rogue-gym as human")
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets your config json file")
                .required(true)
                .takes_value(true),
        )
        .get_matches()
}

fn get_config() -> Result<GameConfig> {
    let args = parse_args();
    let file_name = args.value_of("config").expect("No config file");
    if !file_name.ends_with(".json") {
        return Err(ErrorID::InvalidArg.into_with("Only .json file is allowed as configuration file"));
    }
    let mut file = File::open(file_name).into_chained("get_config")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).into_chained("get_config")?;
    GameConfig::from_json(&buf).map_err(|e| e.convert("get_config"))
}

fn play_game() -> Result<()> {
    let config = get_config()?;
    Ok(())
}
