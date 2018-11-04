extern crate chrono;
extern crate clap;
extern crate fern;
extern crate log;
extern crate rogue_gym_core;
extern crate rogue_gym_devui;
extern crate termion;
extern crate tuple_map;

use std::fs::{File, OpenOptions};
use std::io::{self, Read};

use clap::ArgMatches;
use rogue_gym_core::{json_to_inputs, GameConfig};
use rogue_gym_devui::error::*;
use rogue_gym_devui::{play_game, show_replay};

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
    let (mut config, is_default) = get_config(&args)?;
    if let Some(seed) = args.value_of("seed") {
        config.seed = Some(seed.parse().into_chained(|| "Failed to parse seed!")?);
    }
    setup_logger(&args)?;
    if let Some(fname) = args.value_of("replay") {
        let replay = read_file(fname).into_chained(|| "Failed to read replay file!")?;
        let replay = json_to_inputs(&replay)?;
        let mut interval = 500;
        if let Some(inter) = args.value_of("interval") {
            interval = inter
                .parse()
                .into_chained(|| "Failed to parse 'interval' arg!")?;
        }
        show_replay(config, replay, interval)
    } else {
        play_game(config, is_default)
    }
}

fn read_file(name: &str) -> io::Result<String> {
    let mut file = File::open(name)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn get_config(args: &ArgMatches) -> GameResult<(GameConfig, bool)> {
    let file_name = match args.value_of("config") {
        Some(fname) => fname,
        None => {
            return Ok((GameConfig::default(), true));
        }
    };
    if !file_name.ends_with(".json") {
        return Err(
            ErrorID::InvalidArg.into_with(|| "Only .json file is allowed as configuration file")
        );
    }
    let f = read_file(file_name).into_chained(|| "in get_config")?;
    Ok((GameConfig::from_json(&f)?, false))
}

fn parse_args<'a>() -> ArgMatches<'a> {
    clap::App::new("rogue-gym developper ui")
        .version("0.1.0")
        .author("Yuji Kanagawa <yuji.kngw.80s.revive@gmail.com>")
        .about("play rogue-gym as human")
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets your config json file")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("log")
                .short("l")
                .long("log")
                .value_name("LOG")
                .help("Enable logging to log file")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("filter")
                .short("f")
                .long("filter")
                .value_name("FILTER")
                .help("Set up log level")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("seed")
                .short("s")
                .long("seed")
                .value_name("SEED")
                .help("Set seed")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("replay")
                .short("r")
                .long("replay")
                .value_name("RFILE")
                .help("Open client as replay mode")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("interval")
                .short("i")
                .long("interval")
                .value_name("INTERVAL")
                .help("Interval in replay mode")
                .takes_value(true),
        )
        .get_matches()
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
            })
            .level(level)
            .chain(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(file)
                    .into_chained(|| "error in getting log file")?,
            )
            .apply()
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
