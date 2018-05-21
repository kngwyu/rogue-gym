extern crate chrono;
extern crate clap;
extern crate error_chain_mini;
#[macro_use]
extern crate error_chain_mini_derive;
extern crate fern;
extern crate log;
extern crate rogue_gym_core;
extern crate termion;
extern crate tuple_map;

mod error;
#[macro_use]
mod screen;
use clap::ArgMatches;
use error::{ErrorID, Result};
use error_chain_mini::{ErrorKind, ResultExt};
use rogue_gym_core::dungeon::Positioned;
use rogue_gym_core::ui::{MordalKind, UiState};
use rogue_gym_core::{GameConfig, GameMsg, Reaction, RunTime};
use screen::Screen;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;
use termion::input::TermRead;

fn main() -> Result<()> {
    let args = parse_args();
    let config = get_config(&args)?;
    setup_logger(&args)?;
    play_game(config)
}

fn play_game(config: GameConfig) -> Result<()> {
    let (w, h) = (config.width, config.height);
    let mut screen = Screen::from_stdout(w, h)?;
    screen.welcome()?;
    let mut runtime = config.build().convert()?;
    thread::sleep(Duration::from_secs(1));
    draw_dungeon(&mut screen, &mut runtime)?;
    let stdin = io::stdin();
    // let's receive keyboard inputs(out main loop)
    for keys in stdin.keys() {
        screen.clean_notification()?;
        let key = keys.into_chained("in play_game")?;
        let res = runtime.react_to_key(key.into());
        let res = match res {
            Ok(r) => r,
            Err(e) => {
                // STUB
                notify!(screen, "{}", e.kind().short())?;
                continue;
            }
        };

        for reaction in res {
            let result =
                process_reaction(&mut screen, &mut runtime, reaction).chain_err("in play_game")?;
            if let Some(transition) = result {
                match transition {
                    Transition::Exit => return Ok(()),
                }
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Transition {
    Exit,
}

fn process_reaction(
    screen: &mut Screen,
    runtime: &mut RunTime,
    reaction: Reaction,
) -> Result<Option<Transition>> {
    match reaction {
        Reaction::Notify(msg) => {
            match msg {
                GameMsg::CantMove(_) => Ok(()),
                GameMsg::NoDownStair => notify!(screen, "Hmm... there seems to be no downstair"),
                GameMsg::Quit => {
                    notify!(screen, "Thank you for playing!")?;
                    return Ok(Some(Transition::Exit));
                }
            }.chain_err("in devui::process_reaction")?;
            Ok(None)
        }
        Reaction::Redraw => {
            draw_dungeon(screen, runtime).chain_err("in process_action attempt to draw dungeon")?;
            Ok(None)
        }
        Reaction::UiTransition(ui_state) => {
            if let UiState::Mordal(kind) = ui_state {
                match kind {
                    MordalKind::Quit => notify!(screen, "You really quit game?(y/n)").map(|_| None),
                }
            } else {
                Ok(None)
            }
        }
    }
}

fn draw_dungeon(screen: &mut Screen, runtime: &mut RunTime) -> Result<()> {
    screen.clear_dungeon()?;
    let mut player_pos = None;
    runtime.draw_screen(|Positioned(cd, tile)| {
        if tile.to_byte() == b'@' {
            player_pos = Some(cd);
        }
        screen.draw_tile(cd, tile)
    })?;
    if let Some(cd) = player_pos {
        screen.cursor(cd)?;
    }
    screen.flush()
}

fn get_config(args: &ArgMatches) -> Result<GameConfig> {
    let file_name = args.value_of("config").expect("No config file");
    if !file_name.ends_with(".json") {
        return Err(ErrorID::InvalidArg.into_with("Only .json file is allowed as configuration file"));
    }
    let mut file = File::open(file_name).into_chained("get_config")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).into_chained("get_config")?;
    GameConfig::from_json(&buf).convert()
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
                .help("Sets up log level")
                .takes_value(true),
        )
        .get_matches()
}

fn setup_logger(args: &ArgMatches) -> Result<()> {
    if let Some(file) = args.value_of("log") {
        let level = args.value_of("filter").unwrap_or("debug");
        let level = convert_log_level(level).unwrap_or(log::LevelFilter::Debug);
        use std::str::Chars;
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
            .chain(OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file)
                .into_chained("error in getting log file")?)
            .apply()
            .into_chained("error in setup_log")?;
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
