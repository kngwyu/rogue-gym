use dungeon::{X, Y};
use num_traits::PrimInt;
use rand::{thread_rng, Error as RndError, RngCore, SeedableRng, XorShiftRng};
use rect_iter::IndexError;
use std::collections::BTreeSet;
use std::convert;
use std::ops::Range;
// crate local re-exports
pub(crate) use rand::Rng;
/// Our own ErrorKind type
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

macro_rules! impl_path {
    () => {
        fn as_path(&self) -> &PathImpl {
            &self.inner
        }
        fn as_path_mut(&mut self) -> &mut PathImpl {
            &mut self.inner
        }
        fn from_path(p: PathImpl) -> Self {
            Self { inner: p }
        }
    }
}

/// In the game, we identify all objects by 'path', for dara driven architecture
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct ObjectPath {
    pub(crate) inner: PathImpl,
}

impl Path for ObjectPath {
    impl_path!();
}

/// commonly our dungeon has hierarchy, so
/// use path to specify 'where' is good(though I'm not sure)
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct PlacePath {
    pub(crate) inner: PathImpl,
}

impl Path for PlacePath {
    impl_path!();
}

pub(crate) trait Object {
    fn path(&self) -> ObjectPath;
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct PathImpl(Vec<String>);

pub(crate) trait Path: Sized {
    fn as_path(&self) -> &PathImpl;
    fn as_path_mut(&mut self) -> &mut PathImpl;
    fn from_path(p: PathImpl) -> Self;
    /// construct single path from string
    fn from_str<S: AsRef<str>>(s: S) -> Self {
        let s = s.as_ref().to_owned();
        Self::from_path(PathImpl(vec![s]))
    }
    /// take 'string' and make self 'path::another_path::string'
    fn push<S: AsRef<str>>(&mut self, s: S) {
        let s = s.as_ref().to_owned();
        self.as_path_mut().0.push(s)
    }
    /// concat 2 paths
    fn append(&mut self, mut other: Self) {
        let other = &mut other.as_path_mut().0;
        self.as_path_mut().0.append(other);
    }
}

/// wrapper of XorShiftRng
#[derive(Clone, Serialize, Deserialize)]
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
    /// create new Rng by specified seed
    pub(crate) fn from_seed(seed: u64) -> Self {
        let seed = Self::gen_seed(seed);
        RngHandle(XorShiftRng::from_seed(seed))
    }
    /// create new Rng by random seed
    pub(crate) fn new() -> Self {
        let seed: [u8; 16] = thread_rng().gen();
        RngHandle(XorShiftRng::from_seed(seed))
    }
    /// select some values randomly from given range
    pub(crate) fn select<T: PrimInt>(&mut self, range: Range<T>) -> RngSelect<T> {
        let width = range.end - range.start;
        let width = width.to_u64().expect("[RngHandle::select] NumCast error");
        if width > 10_000_000 {
            panic!("[RngHandle::select] too large range");
        }
        RngSelect {
            current: width,
            offset: range.start,
            selected: (0..width).collect(),
            rng: self,
        }
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

/// Iterator for RngHandle::select
pub(crate) struct RngSelect<'a, T: PrimInt> {
    current: u64,
    offset: T,
    selected: BTreeSet<u64>,
    rng: &'a mut RngHandle,
}

impl<'a, T: PrimInt> Iterator for RngSelect<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.current == 0 {
            return None;
        }
        let r = self.rng.gen_range(0, self.current);
        let res = self.selected
            .iter()
            .nth(r as usize)
            .expect("[RngSelect::Iterator::next] nth returned none(it's a bug!)")
            .to_owned();
        let _ = self.selected.remove(&res);
        self.current -= 1;
        let res = T::from(res).expect("[RngSelect::Iterator::next] NumCast error") + self.offset;
        Some(res)
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
                let plus: u64 = convert::From::from(*byte);
                acc + (plus << (i * 8))
            });
        assert_eq!(seed, decoded);
    }
    #[test]
    fn rng_select() {
        let mut rng = RngHandle::new();
        let v: Vec<_> = rng.select(0..5).take(5).collect();
        assert_eq!(v.len(), 5);
        (0..5).for_each(|i| {
            println!("{}", v[i]);
            assert!(v.iter().find(|&&x| x == i).is_some());
        });
    }
}
