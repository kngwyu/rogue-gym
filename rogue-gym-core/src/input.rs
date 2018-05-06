//! a module for handling user input

/// a representation of Keyboard input
/// almost same as termion::event::Key
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,
}

#[cfg(feature = "termion")]
use error::{ErrorId, GameError};

#[cfg(feature = "termion")]
use termion::event::Key as TermionKey;

#[cfg(feature = "termion")]
impl TryFrom<TermionKey> for Key {
    type Error = GameError;
    fn from(key: TermionKey) -> Result<Key, GameError> {
        use TermionKey::*;
        match key {
            TermionKey::__IsNotComplete => Err(ErrorId::IncompleteInput.into_err()),
            Backspace => Key::Backspace,
            Left => Key::Left,
            Right => Key::Right,
            Up => Key::Up,
            Down => Key::Down,
            Home => Key::Home,
            End => Key::End,
            PageUp => Key::PageUp,
            PageDown => Key::PageDown,
            Delete => Key::Delete,
            Insert => Key::Insert,
            F(x) => Key::F(x),
            Char(x) => Key::Char(x),
            Alt(x) => Key::Alt(x),
            Ctrl(x) => Key::Ctrl(x),
            Null => Key::Null,
            Esc => Key::Esc,
        }
    }
}

#[cfg(feature = "termion")]
impl Into<TermionKey> for Key {
    fn into(self) -> TermionKey {
        use Key::*;
        match key {
            Backspace => TermionKey::Backspace,
            Left => TermionKey::Left,
            Right => TermionKey::Right,
            Up => TermionKey::Up,
            Down => TermionKey::Down,
            Home => TermionKey::Home,
            End => TermionKey::End,
            PageUp => TermionKey::PageUp,
            PageDown => TermionKey::PageDown,
            Delete => TermionKey::Delete,
            Insert => TermionKey::Insert,
            F(x) => TermionKey::F(x),
            Char(x) => TermionKey::Char(x),
            Alt(x) => TermionKey::Alt(x),
            Ctrl(x) => TermionKey::Ctrl(x),
            Null => TermionKey::Null,
            Esc => TermionKey::Esc,
        }
    }
}
