//! a module for handling user input
use dungeon::Direction;
use regex::Regex;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::str;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyMap {
    inner: HashMap<Key, InputCode>,
}

impl Default for KeyMap {
    fn default() -> Self {
        use self::Direction::*;
        use self::InputCode::*;
        let map = vec![
            (Key::Char('l'), Direction(Right)),
            (Key::Char('k'), Direction(Up)),
            (Key::Char('j'), Direction(Down)),
            (Key::Char('h'), Direction(Left)),
            (Key::Char('u'), Direction(RightUp)),
            (Key::Char('y'), Direction(LeftUp)),
            (Key::Char('n'), Direction(RightDown)),
            (Key::Char('b'), Direction(LeftDown)),
            (Key::Up, Direction(Up)),
            (Key::Down, Direction(Down)),
            (Key::Left, Direction(Left)),
            (Key::Right, Direction(Right)),
        ];
        let inner: HashMap<_, _> = map.into_iter().collect();
        KeyMap { inner }
    }
}

impl Serialize for KeyMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.inner.len()))?;
        for (k, v) in &self.inner {
            map.serialize_entry(&k.to_str(), v)?;
        }
        map.end()
    }
}

struct KeyMapVisitor {
    __marker: PhantomData<fn() -> KeyMap>,
}

impl KeyMapVisitor {
    fn new() -> Self {
        KeyMapVisitor {
            __marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for KeyMapVisitor {
    type Value = KeyMap;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("keymap")
    }
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut inner = HashMap::new();
        while let Some((k, v)) = access.next_entry()? {
            let key = match Key::from_str(k) {
                Some(k) => k,
                None => panic!("invalid key name {} in KeyMap", k),
            };
            inner.insert(key, v);
        }
        Ok(KeyMap { inner })
    }
}

impl<'de> Deserialize<'de> for KeyMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(KeyMapVisitor::new())
    }
}

/// generalized user input
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum InputCode {
    Direction(Direction),
}

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

impl Key {
    fn to_str(&self) -> String {
        use self::Key::*;
        match self {
            Backspace => "Backspace".to_owned(),
            Left => "Left".to_owned(),
            Right => "Right".to_owned(),
            Up => "Up".to_owned(),
            Down => "Down".to_owned(),
            Home => "Home".to_owned(),
            End => "End".to_owned(),
            PageUp => "PageUp".to_owned(),
            PageDown => "PageDown".to_owned(),
            Delete => "Delete".to_owned(),
            Insert => "Insert".to_owned(),
            F(u) => format!("F{}", u),
            Char(c) => format!("{}", c),
            Alt(c) => format!("Alt+{}", c),
            Ctrl(c) => format!("Ctrl+{}", c),
            Null => "Null".to_owned(),
            Esc => "Esc".to_owned(),
        }
    }
    fn from_str(s: &str) -> Option<Self> {
        use self::Key::*;
        let f = Regex::new(r"F([0-9]+)").unwrap();
        let alt = Regex::new(r"Alt\s*\+\s*(.+)").unwrap();
        let ctrl = Regex::new(r"Ctrl\s*\+\s*(.+)").unwrap();
        match s {
            "Backspace" => Some(Backspace),
            "Left" => Some(Left),
            "Right" => Some(Right),
            "Up" => Some(Up),
            "Down" => Some(Down),
            "Home" => Some(Home),
            "End" => Some(End),
            "PageUp" => Some(PageUp),
            "PageDown" => Some(PageDown),
            "Delete" => Some(Delete),
            "Insert" => Some(Insert),
            "Null" => Some(Null),
            "Esc" => Some(Esc),
            s if s.len() == 1 => Some(Char(s.chars().nth(0)?)),
            _ => {
                if let Some(cap_f) = f.captures(s) {
                    let num = str::parse::<u8>(&cap_f[1]).ok()?;
                    return Some(F(num));
                }
                if let Some(cap_alt) = alt.captures(s) {
                    return Some(Alt(cap_alt[1].chars().nth(0)?));
                }
                if let Some(cap_ctrl) = ctrl.captures(s) {
                    return Some(Ctrl(cap_ctrl[1].chars().nth(0)?));
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod keymap_test {
    use super::*;
    use serde_json::{from_str, to_string};
    #[test]
    fn from_str_() {
        let f1 = Key::from_str("F1").unwrap();
        assert_eq!(f1, Key::F(1));
        assert_eq!(Key::from_str("FO"), None);
        let alt5 = Key::from_str("Alt+5").unwrap();
        assert_eq!(alt5, Key::Alt('5'));
        let ctrl_a = Key::from_str("Ctrl+a").unwrap();
        assert_eq!(ctrl_a, Key::Ctrl('a'));
        let j = Key::from_str("j").unwrap();
        assert_eq!(j, Key::Char('j'));
    }
    #[test]
    fn serde() {
        let keymap = KeyMap::default();
        let ser = to_string(&keymap).unwrap();
        let de: KeyMap = from_str(&ser).unwrap();
        assert_eq!(de, keymap);
    }
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
