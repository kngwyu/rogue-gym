//! a module for 'path' -- ergonomic object identifier in the game
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
/// In the game, we identify all objects by 'path', for dara driven architecture
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ObjectPath {
    inner: Vec<u32>,
}

impl ObjectPath {
    /// construct single path from string
    pub fn from_str<S: AsRef<str>>(s: S) -> Self {
        let id = intern(s.as_ref());
        ObjectPath { inner: vec![id] }
    }
    /// take 'string' and make self 'path::another_path::string'
    pub fn push<S: AsRef<str>>(&mut self, s: S) {
        let id = intern(s.as_ref());
        self.inner.push(id)
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
            for &id in &self.inner {
                write!(f, "::{}", interner.borrow().get(id).unwrap())?;
            }
            writeln!(f, "")
        })
    }
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::new());
}

fn intern(string: &str) -> u32 {
    INTERNER.with(|interner| interner.borrow_mut().intern(string))
}

fn get_owned(id: u32) -> Option<String> {
    INTERNER.with(|interner| interner.borrow().get(id).map(|s| s.to_owned()))
}

#[derive(Default)]
struct Interner {
    ids: HashMap<Box<str>, u32>,
    strings: Vec<Box<str>>,
}

impl Interner {
    fn new() -> Self {
        Self::default()
    }
    fn intern(&mut self, string: &str) -> u32 {
        if let Some(&id) = self.ids.get(string) {
            return id;
        }
        let id = self.strings.len() as u32;
        let string = string.to_string().into_boxed_str();
        self.strings.push(string.clone());
        self.ids.insert(string, id);
        id
    }
    fn get(&self, id: u32) -> Option<&str> {
        match self.strings.get(id as usize) {
            Some(ref s) => Some(s),
            None => None,
        }
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
