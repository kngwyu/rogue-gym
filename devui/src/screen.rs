use error::*;
use rogue_gym_core::{character::player, dungeon::Coord, tile::Tile};
use std::io::{self, Stdout, Write};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor, terminal_size};
use tuple_map::TupleMap2;

/// wrapper of stdout as rogue screen
pub struct Screen<T> {
    /// stdout
    pub term: T,
    pub has_notification: bool,
    width: u16,
    height: u16,
}

impl Screen<RawTerminal<Stdout>> {
    /// raw terminal screen(for cli)
    pub fn from_raw(w: i32, h: i32) -> GameResult<Self> {
        let stdout = io::stdout();
        let term = stdout
            .into_raw_mode()
            .into_chained(|| "[Screen::from_stdout] attempt to get raw mode terminal")?;
        let (width, height) = terminal_size()
            .into_chained(|| "[Screen::from_stdout] attempt to get terminal size")?;
        let (w, h) = (w, h).map(|i| i as u16);
        if width < w {
            return Err(ErrorID::InvalidScreenSize(width, height)
                .into_with(|| format!("Screen width must be larger than {} characters", w)));
        }
        if height < h {
            return Err(ErrorID::InvalidScreenSize(width, height)
                .into_with(|| format!("Screen height must be larger than {} characters", h)));
        }
        Ok(Screen {
            term,
            has_notification: false,
            width: w,
            height: h,
        })
    }
}

impl Screen<Stdout> {
    /// raw terminal screen(for python API)
    pub fn from_stdout(w: i32, h: i32) -> GameResult<Self> {
        let stdout = io::stdout();
        let (width, height) = terminal_size()
            .into_chained(|| "[Screen::from_stdout] attempt to get terminal size")?;
        let (w, h) = (w, h).map(|i| i as u16);
        if width < w {
            return Err(ErrorID::InvalidScreenSize(width, height)
                .into_with(|| format!("Screen width must be larger than {} characters", w)));
        }
        if height < h {
            return Err(ErrorID::InvalidScreenSize(width, height)
                .into_with(|| format!("Screen height must be larger than {} characters", h)));
        }
        Ok(Screen {
            term: stdout,
            has_notification: false,
            width,
            height,
        })
    }
}

impl<T: Write> Screen<T> {
    pub fn clear_dungeon(&mut self) -> GameResult<()> {
        (2..self.height)
            .try_for_each(|row| write!(self.term, "{}{}", cursor::Goto(1, row), clear::CurrentLine))
            .into_chained(|| "in Screen::clear")
    }

    pub fn clear_notification(&mut self) -> GameResult<()> {
        if self.has_notification {
            self.has_notification = false;
            write!(
                self.term,
                "{}{}",
                ::termion::cursor::Goto(1, 1),
                ::termion::clear::CurrentLine
            )
        } else {
            Ok(())
        }
        .into_chained(|| "in Screen::clean_notification")
    }

    pub fn cursor<P: Into<(u16, u16)>>(&mut self, cd: P) -> GameResult<()> {
        let (col, row) = cd.into();
        write!(self.term, "{}", cursor::Goto(col, row)).into_chained(|| "in Screen::draw_tile")
    }
    pub fn draw_tile(&mut self, cd: Coord, tile: Tile) -> GameResult<()> {
        let (col, row) = cd.into();
        write!(self.term, "{}{}", cursor::Goto(col, row), tile.to_char())
            .into_chained(|| "in Screen::draw_tile")
    }

    pub fn flush(&mut self) -> GameResult<()> {
        self.term.flush().into_chained(|| "Screen::flush")
    }

    pub fn welcome(&mut self) -> GameResult<()> {
        write!(
            self.term,
            "{}{} Welcome to rogue-gym!{}Wait a minute while we're digging the dungeon...",
            clear::All,
            cursor::Goto(1, 1),
            cursor::Goto(1, 2)
        )
        .into_chained(|| "in Screen::welcome")?;
        self.flush().chain_err(|| "in Screen::welcome")
    }

    pub fn default_config(&mut self) -> GameResult<()> {
        write!(
            self.term,
            "{} No config file is specified, use default settings",
            cursor::Goto(1, 3),
        )
        .into_chained(|| "in Screen::default_config")?;
        self.flush().chain_err(|| "in Screen::default_config")
    }

    pub fn status(&mut self, status: player::Status) -> GameResult<()> {
        write!(
            self.term,
            "{}{}{}",
            cursor::Goto(1, self.height),
            clear::CurrentLine,
            status,
        )
        .into_chained(|| "in Screen::status")?;
        self.flush().chain_err(|| "in Screen::status")
    }
}

#[macro_export]
macro_rules! notify {
    ($screen: ident, $msg: expr) => {{
        $screen.has_notification = true;
        if let Err(e) = write!($screen.term, "{}{}", ::termion::cursor::Goto(1, 1), ::termion::clear::CurrentLine) {
            Err(e).into_chained(||"in notify!")
        } else {
            if let Err(e) = write!($screen.term, $msg) {
                Err(e).into_chained(||"in notify!")
            } else {
                $screen.flush()
            }
        }
    }};
    ($screen: ident, $fmt: expr, $($arg: tt)*) => {{
        $screen.has_notification = true;
        if let Err(e) = write!($screen.term, "{}{}", ::termion::cursor::Goto(1, 1), ::termion::clear::CurrentLine) {
            Err(e).into_chained(||"in notify!")
        } else {
            if let Err(e) = write!($screen.term, $fmt, $($arg)*) {
                Err(e).into_chained(||"in notify!")
            } else {
                $screen.flush()
            }
        }
    }};
}
