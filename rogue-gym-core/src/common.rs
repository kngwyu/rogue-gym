use dungeon::{X, Y};
use rand::{thread_rng, Error as RndError, RngCore, SeedableRng, XorShiftRng};
use rect_iter::IndexError;

// crate local re-exports
pub(crate) use rand::Rng;
#[derive(Clone, Debug, ErrorKind)]
pub enum ErrorId {
    #[msg(short = "Invalid index access", detailed = "x: {:?}, y: {:?}", x, y)]
    Index { x: Option<X>, y: Option<Y> },
}

impl From<IndexError> for ErrorId {
    fn from(e: IndexError) -> Self {
        ErrorId::Index {
            x: Some(X(e.x as i32)),
            y: Some(Y(e.y as i32)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ObjectPath(Vec<String>);

impl ObjectPath {
    pub fn from_str<S: AsRef<str>>(s: S) -> Self {
        let s = s.as_ref().to_owned();
        ObjectPath(vec![s])
    }
    pub fn push<S: AsRef<str>>(&mut self, s: S) {
        let s = s.as_ref().to_owned();
        self.0.push(s)
    }
    pub fn append(&mut self, mut other: ObjectPath) {
        self.0.append(&mut other.0);
    }
}

pub trait Object {
    fn path(&self) -> ObjectPath;
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RngHandle(XorShiftRng);

impl RngHandle {
    fn gen_seed(seed: u64) -> [u8; 16] {
        let mut seed_bytes = [0u8; 16];
        for i in 0..8 {
            let shift = i * 8;
            let val = (seed >> shift) as u8;
            seed_bytes[i] = val;
            seed_bytes[15 - i] = val;
        }
        seed_bytes
    }
    pub(crate) fn from_seed(seed: u64) -> Self {
        let seed = Self::gen_seed(seed);
        RngHandle(XorShiftRng::from_seed(seed))
    }
    pub(crate) fn new() -> Self {
        let seed: [u8; 16] = thread_rng().gen();
        RngHandle(XorShiftRng::from_seed(seed))
    }
}

impl RngCore for RngHandle {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }
    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }
    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RndError> {
        self.0.try_fill_bytes(dest)
    }
}

#[cfg(test)]
mod rng_test {
    use super::*;
    /// seed encoding test
    #[test]
    fn gen_seed() {
        let seed = 2367689;
        let seed_bytes = RngHandle::gen_seed(seed);
        let decoded = seed_bytes
            .into_iter()
            .take(8)
            .enumerate()
            .fold(0, |acc, (i, byte)| {
                let plus = u64::from(*byte) << (i * 8);
                acc + plus
            });
        assert_eq!(seed, decoded);
    }
}
