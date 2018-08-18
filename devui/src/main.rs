extern crate chrono;
extern crate clap;
extern crate fern;
extern crate log;
extern crate rogue_gym_core;
extern crate rogue_gym_devui;
extern crate termion;
extern crate tuple_map;

use std::fs::{File, OpenOptions};
use std::io::Read;

use clap::ArgMatches;
use rogue_gym_core::GameConfig;
use rogue_gym_devui::error::*;
use rogue_gym_devui::play_game;

fn main() {
    if let Err(err) = main_() {
        eprintln!("Oops! Error occured in rogue-gym-devui:");
        let errs: Vec<_> = err.iter_chain().map(|e| format!("  {}", e)).collect();
        for e in errs.into_iter().rev() {
            eprintln!("{}", e);
        }
        ::std::process::exit(1);
    }
}

fn main_() -> GameResult<()> {
    let args = parse_args();
    let config = get_config(&args)?;
    setup_logger(&args)?;
    play_game(config)
}

fn get_config(args: &ArgMatches) -> GameResult<GameConfig> {
    let file_name = args.value_of("config").expect("No config file");
    if !file_name.ends_with(".json") {
        return Err(
            ErrorID::InvalidArg.into_with(|| "Only .json file is allowed as configuration file")
        );
    }
    let mut file = File::open(file_name).into_chained(|| "get_config")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .into_chained(|| "get_config")?;
    GameConfig::from_json(&buf)
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
        ).arg(
            clap::Arg::with_name("log")
                .short("l")
                .long("log")
                .value_name("LOG")
                .help("Enable logging to log file")
                .takes_value(true),
        ).arg(
            clap::Arg::with_name("filter")
                .short("f")
                .long("filter")
                .value_name("FILTER")
                .help("Sets up log level")
                .takes_value(true),
        ).get_matches()
}

fn setup_logger(args: &ArgMatches) -> GameResult<()> {
    if let Some(file) = args.value_of("log") {
        let level = args.value_of("filter").unwrap_or("debug");
        let level = convert_log_level(level).unwrap_or(log::LevelFilter::Debug);
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            }).level(level)
            .chain(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(file)
                    .into_chained(|| "error in getting log file")?,
            ).apply()
            .into_chained(|| "error in setup_log")?;
    }
    Ok(())
}

fn convert_log_level(s: &str) -> Option<log::LevelFilter> {
    use log::LevelFilter::*;
    let s = s.to_lowercase();
    match &*s {
        "off" | "o" => Some(Off),
        "error" | "e" => Some(Error),
        "warn" | "w" => Some(Warn),
        "info" | "i" => Some(Info),
        "debug" | "d" => Some(Debug),
        "trace" | "t" => Some(Trace),
        _ => None,
    }
}
