use error::*;
use rogue_gym_core::dungeon::{Coord, X, Y};
use rogue_gym_uilib::Screen;
use std::collections::VecDeque;
use std::io::{self, Stdout, Write};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor, terminal_size};
use tuple_map::TupleMap2;

pub type RawTerm = RawTerminal<Stdout>;

/// wrapper of stdout as rogue screen
pub struct TermScreen<T> {
    /// stdout
    term: T,
    has_notification: bool,
    width: u16,
    height: u16,
    pub(crate) pending_messages: VecDeque<String>,
}

impl TermScreen<RawTerm> {
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
        Ok(TermScreen {
            term,
            has_notification: false,
            width: w,
            height: h,
            pending_messages: VecDeque::new(),
        })
    }
}

impl TermScreen<Stdout> {
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
        Ok(TermScreen {
            term: stdout,
            has_notification: false,
            width,
            height,
            pending_messages: VecDeque::new(),
        })
    }
}

impl<T: Write> Screen for TermScreen<T> {
    fn width(&self) -> X {
        X(i32::from(self.width))
    }
    fn height(&self) -> Y {
        Y(i32::from(self.height))
    }
    fn message<S: AsRef<str>>(&mut self, msg: S) -> GameResult<()> {
        self.clear_line(Y(0))?;
        self.has_notification = true;
        self.write_str(Coord::new(0, 0), msg.as_ref())
    }
    fn clear_line(&mut self, row: Y) -> GameResult<()> {
        let row = row.0 as u16;
        write!(self.term, "{}{}", cursor::Goto(1, row), clear::CurrentLine)
            .into_chained(|| "in TermScreen::clear_line")
    }
    fn clear_notification(&mut self) -> GameResult<()> {
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
        .into_chained(|| "in TermScreen::clear_notification")
    }
    fn cursor(&mut self, coord: Coord) -> GameResult<()> {
        write!(self.term, "{}", coord.into_cursor()).into_chained(|| "in TermScreen::cursor")
    }
    fn flush(&mut self) -> GameResult<()> {
        self.term.flush().into_chained(|| "in TermScreen::flush")
    }
    fn write_char(&mut self, cd: Coord, c: char) -> GameResult<()> {
        write!(self.term, "{}{}", cd.into_cursor(), c).into_chained(|| "in TermScreen::write_char")
    }
    fn write_str<S: AsRef<str>>(&mut self, start: Coord, s: S) -> GameResult<()> {
        write!(
            self.term,
            "{}{}{}",
            start.into_cursor(),
            clear::CurrentLine,
            s.as_ref()
        )
        .into_chained(|| "in TermScreen::write_str")?;
        self.flush().chain_err(|| "in TermScreen::write_str")
    }
    fn pend_message<S: AsRef<str>>(&mut self, msg: S) -> GameResult<()> {
        self.pending_messages.push_back(msg.as_ref().to_owned());
        Ok(())
    }
}

impl<T: Write> TermScreen<T> {
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
    pub fn display_msg(&mut self) -> GameResult<bool> {
        if let Some(msg) = self.pending_messages.pop_front() {
            if self.pending_messages.is_empty() {
                self.message(msg)?;
                Ok(false)
            } else {
                self.message(msg + "--More--")?;
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}
