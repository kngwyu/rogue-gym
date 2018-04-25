/// Keycode
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
use termion::event::Key as TermionKey;

#[cfg(feature = "termion")]
impl From<TermionKey> for Key {}
