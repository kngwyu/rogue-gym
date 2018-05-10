//! a module for 'path' -- ergonomic object identifier in the game
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

type Symbol = u16;

/// In the game, we identify all objects by 'path', for dara driven architecture
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ObjectPath {
    inner: Vec<Symbol>,
}

impl ObjectPath {
    /// construct single path from string
    pub fn from_str<S: AsRef<str>>(s: S) -> Self {
        let sym = intern(s.as_ref());
        ObjectPath { inner: vec![sym] }
    }
    /// take 'string' and make self 'path::another_path::string'
    pub fn push<S: AsRef<str>>(&mut self, s: S) {
        let sym = intern(s.as_ref());
        self.inner.push(sym)
    }
    /// concat 2 paths
    pub fn append(&mut self, mut other: Self) {
        self.inner.append(&mut other.inner);
    }
}

impl Serialize for ObjectPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        OwnedPath::from_path(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ObjectPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        OwnedPath::deserialize(deserializer).map(|owned| owned.into_path())
    }
}

#[derive(Serialize, Deserialize)]
struct OwnedPath {
    inner: Vec<String>,
}

impl OwnedPath {
    fn from_path(path: &ObjectPath) -> Self {
        let res: Vec<_> = path.inner
            .iter()
            .map(|&u| get_owned(u).expect("[ObjectPath::get_owned] Invalid Path"))
            .collect();
        OwnedPath { inner: res }
    }
    fn into_path(self) -> ObjectPath {
        ObjectPath {
            inner: self.inner.into_iter().map(|s| intern(&s)).collect(),
        }
    }
}

impl fmt::Display for ObjectPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        INTERNER.with(|interner| {
            for &sym in &self.inner {
                write!(f, "::{}", interner.borrow().get(sym).unwrap())?;
            }
            Ok(())
        })
    }
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::new());
}

fn intern(string: &str) -> Symbol {
    INTERNER.with(|interner| interner.borrow_mut().intern(string))
}

fn get_owned(sym: Symbol) -> Option<String> {
    INTERNER.with(|interner| interner.borrow().get(sym).map(|s| s.to_owned()))
}

#[derive(Default)]
struct Interner {
    ids: HashMap<Box<str>, Symbol>,
    strings: Vec<Box<str>>,
}

impl Interner {
    fn new() -> Self {
        Self::default()
    }
    fn intern(&mut self, string: &str) -> Symbol {
        if let Some(&sym) = self.ids.get(string) {
            return sym;
        }
        let sym = self.strings.len() as Symbol;
        let string = string.to_string().into_boxed_str();
        self.strings.push(string.clone());
        self.ids.insert(string, sym);
        sym
    }
    fn get(&self, sym: Symbol) -> Option<&str> {
        self.strings.get(sym as usize).map(|s| s.as_ref())
    }
}

#[test]
fn interner_test() {
    let a = "apple";
    let b = "asdhfsll";
    let a_id = intern(a);
    assert_eq!(a_id, 0);
    let b_id = intern(b);
    assert_eq!(b_id, 1);
    assert_eq!(get_owned(a_id).unwrap(), "apple");
    assert_eq!(intern(a), 0);
}
