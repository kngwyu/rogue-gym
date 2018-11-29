use serde::{
    de::Error, de::Unexpected, de::Visitor, Deserialize, Deserializer, Serialize, Serializer,
};
use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;
use std::str;

#[derive(Clone, Default)]
pub struct SmallStr(Repr);

const MAX_SHORT_LEN: usize = 15;

#[derive(Clone, Debug)]
enum Repr {
    Inline([u8; MAX_SHORT_LEN], u8),
    Heap(Box<str>),
}

impl Default for Repr {
    fn default() -> Self {
        Repr::Inline([0u8; MAX_SHORT_LEN], 0)
    }
}

impl SmallStr {
    pub fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len();
        if len <= MAX_SHORT_LEN {
            let mut data = [0u8; MAX_SHORT_LEN];
            data[0..len].clone_from_slice(bytes);
            SmallStr(Repr::Inline(data, len as u8))
        } else {
            SmallStr(Repr::Heap(s.to_owned().into_boxed_str()))
        }
    }
    pub fn from_string(s: String) -> Self {
        let len = s.as_bytes().len();
        if len <= MAX_SHORT_LEN {
            let mut data = [0u8; MAX_SHORT_LEN];
            data[0..len].clone_from_slice(s.as_bytes());
            SmallStr(Repr::Inline(data, len as u8))
        } else {
            SmallStr(Repr::Heap(s.into_boxed_str()))
        }
    }
    pub fn into_string(self) -> String {
        match self.0 {
            Repr::Heap(s) => String::from(s),
            Repr::Inline(s, len) => unsafe {
                String::from_utf8_unchecked(s[..usize::from(len)].to_owned())
            },
        }
    }
    fn from_bytes(v: Vec<u8>) -> Result<Self, Vec<u8>> {
        let len = v.len();
        if len <= MAX_SHORT_LEN {
            match str::from_utf8(&v) {
                Ok(_) => {
                    let mut data = [0u8; MAX_SHORT_LEN];
                    data[0..len].clone_from_slice(&v);
                    Ok(SmallStr(Repr::Inline(data, len as u8)))
                }
                Err(_) => Err(v),
            }
        } else {
            match String::from_utf8(v) {
                Ok(s) => Ok(SmallStr(Repr::Heap(s.into_boxed_str()))),
                Err(e) => Err(e.into_bytes()),
            }
        }
    }
    pub fn as_str(&self) -> &str {
        match self.0 {
            Repr::Heap(ref s) => s.as_ref(),
            Repr::Inline(ref s, len) => unsafe {
                &str::from_utf8_unchecked(&s[..usize::from(len)])
            },
        }
    }
}

impl AsRef<str> for SmallStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for SmallStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for SmallStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl PartialEq<str> for SmallStr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for SmallStr {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<SmallStr> for SmallStr {
    fn eq(&self, other: &SmallStr) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&SmallStr> for SmallStr {
    fn eq(&self, other: &&SmallStr) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for SmallStr {}

impl PartialOrd<str> for SmallStr {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialOrd<SmallStr> for SmallStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for SmallStr {
    fn cmp(&self, other: &SmallStr) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Serialize for SmallStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SmallStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(SmallStrVisitor::new())
    }
}

struct SmallStrVisitor {
    __marker: PhantomData<fn() -> SmallStr>,
}

impl SmallStrVisitor {
    fn new() -> Self {
        SmallStrVisitor {
            __marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for SmallStrVisitor {
    type Value = SmallStr;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a small string")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(SmallStr::from_str(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(SmallStr::from_string(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match str::from_utf8(v) {
            Ok(s) => Ok(SmallStr::from_str(s)),
            Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match SmallStr::from_bytes(v) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::invalid_value(Unexpected::Bytes(&e), &self)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::SmallStr;
    #[test]
    fn from_str() {
        let s = SmallStr::from_str("apple");
        assert_eq!(s, "apple");
        let s = SmallStr::from_str("Bigmouth strikes again");
        assert_eq!(s, "Bigmouth strikes again");
    }
    #[test]
    fn from_string() {
        let s = SmallStr::from_string("apple".to_owned());
        assert_eq!(s, "apple");
        let s = SmallStr::from_string("Bigmouth strikes again".to_owned());
        assert_eq!(s, "Bigmouth strikes again");
    }
}
