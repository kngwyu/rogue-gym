//! UI abstraction for rogue-gym
use rogue_gym_core::dungeon::{Coord, Positioned, X, Y};
use rogue_gym_core::error::GameResult;
use rogue_gym_core::ui::{MordalKind, UiState};
use rogue_gym_core::{character::player::Status, tile::Tile, GameMsg, Reaction, RunTime};

/// 0-indexed 2d screen for rogue-gym
pub trait Screen {
    fn width(&self) -> X;
    fn height(&self) -> Y;
    fn clear_line(&mut self, row: Y) -> GameResult<()>;
    fn clear_dungeon(&mut self) -> GameResult<()> {
        (1..self.height().0 - 1).try_for_each(|row| self.clear_line(Y(row)))
    }
    fn clear_screen(&mut self) -> GameResult<()> {
        (2..self.height().0 - 1).try_for_each(|row| self.clear_line(Y(row)))
    }
    fn clear_notification(&mut self) -> GameResult<()> {
        self.clear_line(self.height() - 1.into())
    }
    fn cursor(&mut self, _cursor: Coord) -> GameResult<()> {
        Ok(())
    }
    fn flush(&mut self) -> GameResult<()> {
        Ok(())
    }
    fn write_char(&mut self, pos: Coord, c: char) -> GameResult<()>;
    fn write_tile(&mut self, pos: Coord, t: Tile) -> GameResult<()> {
        self.write_char(pos, t.to_char())
    }
    fn write_str<S: AsRef<str>>(&mut self, start: Coord, s: S) -> GameResult<()> {
        let mut current = start;
        for c in s.as_ref().chars() {
            self.write_char(current, c)?;
            current.x.0 += 1;
        }
        self.flush()
    }
    fn message<S: AsRef<str>>(&mut self, msg: S) -> GameResult<()> {
        self.clear_line(Y(0))?;
        self.write_str(Coord::new(0, 0), msg.as_ref())
    }
    fn pend_message<S: AsRef<str>>(&mut self, msg: S) -> GameResult<()> {
        self.message(msg)
    }
    fn status(&mut self, status: &Status) -> GameResult<()> {
        self.write_str(
            Coord::new(0, self.height() - 1.into()),
            format!("{}", status),
        )
    }
    fn dungeon(&mut self, runtime: &mut RunTime) -> GameResult<()> {
        let mut player_pos = None;
        runtime.draw_screen(|Positioned(cd, tile)| {
            if tile.to_byte() == b'@' {
                player_pos = Some(cd);
            }
            self.write_tile(cd, tile)
        })?;
        if let Some(pos) = player_pos {
            self.cursor(pos)?;
        }
        self.flush()
    }
    fn inventory(&mut self, runtime: &mut RunTime) -> GameResult<()> {
        for (i, item) in runtime.itembox().items().enumerate() {
            let num = (b'a' + i as u8) as char;
            self.write_str(Coord::new(0, i as i32), format!("{}) {}", num, item))?;
        }
        self.write_str(
            Coord::new(0, self.height() - 1.into()),
            "--Press space to continue--",
        )
    }
    fn dying_msg(&mut self, sig: &str) -> GameResult<()> {
        const MESSAGES: [&'static str; 9] = [
            r"                __________        ",
            r"               /          \       ",
            r"              /    REST    \      ",
            r"             /     IN       \     ",
            r"            /     PEACE      \    ",
            r"           /                  \   ",
            r"           |                  |   ",
            r"          *|     *  *  *      |   *",
            r" ________)/\\_//(\/(/\)/\//\/|_)_______",
        ];
        let sig = if sig.len() > 18 { &sig[..18] } else { sig };
        for (i, msg) in MESSAGES.iter().enumerate() {
            if i == 6 {
                let mut s = format!("           |{}", sig);
                for _ in 0..18 - sig.len() {
                    s.push_str(" ")
                }
                s.push_str("|");
                let _ = self.write_str(Coord::new(0, i as i32 + 2), s);
            } else {
                let _ = self.write_str(Coord::new(0, i as i32 + 2), msg);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transition {
    Exit,
    None,
}

pub fn process_reaction<S: Screen>(
    screen: &mut S,
    runtime: &mut RunTime,
    reaction: Reaction,
) -> GameResult<Transition> {
    match reaction {
        Reaction::Notify(msg) => match msg {
            GameMsg::CantMove(_) => Ok(()),
            GameMsg::CantGetItem(kind) => screen.pend_message(format!("You walk onto {:?}", kind)),
            GameMsg::NoDownStair => {
                screen.pend_message(format!("Hmm... there seems to be no downstair"))
            }
            GameMsg::GotItem { kind, num } => {
                screen.pend_message(format!("You got {} {:?}", num, kind))
            }
            GameMsg::SecretDoor => screen.pend_message(format!("You found a secret door")),
            GameMsg::HitTo(s) => screen.pend_message(format!("You swings and hit {}", s)),
            GameMsg::HitFrom(s) => screen.pend_message(format!("{} swings and hits you", s)),
            GameMsg::MissTo(s) => screen.pend_message(format!("You swing and miss {}", s)),
            GameMsg::MissFrom(s) => screen.pend_message(format!("{} swings and misses you", s)),
            GameMsg::Killed(s) => screen.pend_message(format!("You defeated the {}", s)),
            GameMsg::Quit => {
                screen.pend_message(format!("Thank you for playing!"))?;
                return Ok(Transition::Exit);
            }
        },
        Reaction::Redraw => screen.dungeon(runtime),
        Reaction::StatusUpdated => screen.status(&runtime.player_status()),
        Reaction::UiTransition(ui_state) => match ui_state {
            UiState::Mordal(kind) => match kind {
                MordalKind::Quit => screen.message(format!("You really quit game?(y/n)")),
                MordalKind::Inventory => screen.inventory(runtime),
                MordalKind::Grave(msg) => screen.dying_msg(&*msg),
            },
            UiState::Dungeon => {
                screen.dungeon(runtime)?;
                screen.clear_line(0.into())?;
                screen.status(&runtime.player_status())
            }
        },
    }?;
    Ok(Transition::None)
}
