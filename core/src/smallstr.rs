use serde::{Deserialize, Serialize};
use std::ptr;

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SmallStr(Repr);

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum Repr {
    Inline([u8; 16]),
    Heap(Box<str>),
}

impl Default for Repr {
    fn default() -> Self {
        Repr::Inline(inline())
    }
}

fn inline() -> [u8; 16] {
    [0u8; 16]
}

impl SmallStr {
    pub fn new(s: &str) -> Self {
        let bytes = s.as_bytes();
        if bytes.len() <= 16 {
            let mut data = inline();
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), data.as_mut_ptr(), bytes.len());
            }
            SmallStr(Repr::Inline(data))
        } else {
            SmallStr(Repr::Heap(s.to_owned().into_boxed_str()))
        }
    }
}
