extern crate chrono;
extern crate clap;
#[macro_use]
extern crate failure;
extern crate fern;
#[macro_use]
extern crate log;
extern crate rogue_gym_core;
extern crate termion;
extern crate tuple_map;

pub mod error;
#[macro_use]
pub mod screen;
use error::*;
use rogue_gym_core::dungeon::Positioned;
use rogue_gym_core::input::InputCode;
use rogue_gym_core::ui::{MordalKind, UiState};
use rogue_gym_core::{GameConfig, GameMsg, Reaction, RunTime};
use screen::{RawTerm, Screen};
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;

fn setup_screen(config: GameConfig, is_default: bool) -> GameResult<(Screen<RawTerm>, RunTime)> {
    let mut screen = Screen::from_raw(config.width, config.height)?;
    screen.welcome()?;
    if is_default {
        screen.default_config()?;
    }
    let mut runtime = config.build()?;
    thread::sleep(Duration::from_secs(1));
    draw_dungeon(&mut screen, &mut runtime)?;
    screen.status(runtime.player_status())?;
    Ok((screen, runtime))
}

pub fn play_game(config: GameConfig, is_default: bool) -> GameResult<()> {
    debug!("devui::play_game config: {:?}", config);
    let (mut screen, mut runtime) = setup_screen(config, is_default)?;
    let stdin = io::stdin();
    // let's receive keyboard inputs(our main loop)
    for keys in stdin.keys() {
        screen.clear_notification()?;
        let key = keys.into_chained(|| "in play_game")?;
        let res = runtime.react_to_key(key.into());
        let res = match res {
            Ok(r) => r,
            Err(e) => {
                // STUB
                notify!(screen, "{}", e)?;
                continue;
            }
        };
        for reaction in res {
            let result = process_reaction(&mut screen, &mut runtime, reaction)
                .chain_err(|| "in play_game")?;
            if let Some(transition) = result {
                match transition {
                    Transition::Exit => return Ok(()),
                }
            }
        }
    }
    Ok(())
}

pub fn show_replay(config: GameConfig, replay: Vec<InputCode>, interval_ms: u64) -> GameResult<()> {
    debug!("devui::show_replay config: {:?}", config);
    let (tx, rx) = mpsc::channel();
    let replay_thread = thread::spawn(move || {
        let res = show_replay_(config, replay, interval_ms, rx);
        if let Err(e) = res {
            eprintln!("Error in viewer: {}", e);
        }
    });
    let stdin = io::stdin();
    for key in stdin.keys() {
        let key = key.into_chained(|| "in show_replay")?;
        let mut end = false;
        let res = match key {
            Key::Char('e') | Key::Char('q') | Key::Esc => {
                end = true;
                tx.send(ReplayInst::End)
            }
            Key::Char('p') => tx.send(ReplayInst::Pause),
            Key::Char('s') => tx.send(ReplayInst::Start),
            _ => continue,
        };
        if let Err(e) = res {
            eprintln!("Error in viewer: {}", e);
        }
        if end {
            break;
        }
    }
    replay_thread.join().unwrap();
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum ReplayInst {
    Pause,
    Start,
    End,
}

fn show_replay_(
    config: GameConfig,
    replay: Vec<InputCode>,
    interval_ms: u64,
    rx: mpsc::Receiver<ReplayInst>,
) -> GameResult<()> {
    let (mut screen, mut runtime) = setup_screen(config, false)?;
    let mut sleeping = false;
    let mut replay_idx = 0;
    loop {
        match rx.try_recv() {
            Ok(ReplayInst::Start) => sleeping = false,
            Ok(ReplayInst::Pause) => sleeping = true,
            Ok(ReplayInst::End) => return Ok(()),
            Err(mpsc::TryRecvError::Disconnected) => bail!("devui::show_replay disconnected!"),
            Err(mpsc::TryRecvError::Empty) => {}
        }
        thread::sleep(Duration::from_millis(interval_ms));
        if sleeping {
            continue;
        }
        let input = match replay.get(replay_idx) {
            Some(x) => *x,
            None => continue,
        };
        replay_idx += 1;
        let res = runtime.react_to_input(input);
        let res = match res {
            Ok(r) => r,
            Err(e) => {
                notify!(screen, "{}", e)?;
                continue;
            }
        };
        for reaction in res {
            let result = process_reaction(&mut screen, &mut runtime, reaction)
                .chain_err(|| "in show_replay")?;
            if let Some(transition) = result {
                match transition {
                    Transition::Exit => return Ok(()),
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transition {
    Exit,
}

pub fn process_reaction<W: Write>(
    screen: &mut Screen<W>,
    runtime: &mut RunTime,
    reaction: Reaction,
) -> GameResult<Option<Transition>> {
    match reaction {
        Reaction::Notify(msg) => {
            match msg {
                // GameMsg::CantMove(d) => notify!(screen, "your {} way is blocked", d),
                GameMsg::CantMove(_) => Ok(()),
                // TODO: Display for ItemKind
                GameMsg::CantGetItem(kind) => notify!(screen, "You walk onto {:?}", kind),
                GameMsg::NoDownStair => notify!(screen, "Hmm... there seems to be no downstair"),
                GameMsg::GotItem { kind, num } => {
                    notify!(screen, "Now you have {} {:?}", num, kind)
                }
                GameMsg::SecretDoor => notify!(screen, "you found a secret door"),
                GameMsg::Quit => {
                    notify!(screen, "Thank you for playing!")?;
                    return Ok(Some(Transition::Exit));
                }
            }
            .chain_err(|| "in devui::process_reaction")?;
            Ok(None)
        }
        Reaction::Redraw => {
            draw_dungeon(screen, runtime)
                .chain_err(|| "in process_action attempt to draw dungeon")?;
            Ok(None)
        }
        Reaction::StatusUpdated => {
            let status = runtime.player_status();
            screen.status(status)?;
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

pub fn draw_dungeon<W: Write>(screen: &mut Screen<W>, runtime: &mut RunTime) -> GameResult<()> {
    screen.clear_dungeon()?;
    let mut player_pos = None;
    runtime.draw_screen(|Positioned(cd, tile)| {
        if tile.to_byte() == b'@' {
            player_pos = Some(cd);
        }
        screen.draw_tile(cd, tile)
    })?;
    if let Some(pos) = player_pos {
        screen.cursor(pos)?;
    }
    screen.flush()
}
