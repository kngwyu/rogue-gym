use num_traits::PrimInt;
pub(crate) use rand::Rng;
use rand::{distributions::uniform::SampleUniform, thread_rng, Error as RndError, RngCore,
           SeedableRng, XorShiftRng};
use std::cell::UnsafeCell;
use std::collections::BTreeSet;
use std::convert;
use std::iter::FromIterator;
use std::ops::Range;

/// wrapper of XorShiftRng
#[derive(Clone, Serialize, Deserialize)]
pub struct RngHandle(XorShiftRng);

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
    pub fn from_seed(seed: u64) -> Self {
        let seed = Self::gen_seed(seed);
        RngHandle(XorShiftRng::from_seed(seed))
    }
    /// create new Rng by random seed
    pub fn new() -> Self {
        let seed: [u8; 16] = thread_rng().gen();
        RngHandle(XorShiftRng::from_seed(seed))
    }
    /// select some values randomly from given range
    pub fn select<T: PrimInt>(&mut self, range: Range<T>) -> RandomSelecter<T> {
        let width = range.end - range.start;
        let width = width.to_u64().expect("[RngHandle::select] NumCast error");
        if width > 10_000_000 {
            panic!("[RngHandle::select] too large range");
        }
        RandomSelecter {
            current: width,
            offset: range.start,
            selected: (0..width).collect(),
            rng: self,
        }
    }
    /// wrapper of gen_range which takes Range
    pub fn range<T>(&mut self, range: Range<T>) -> T
    where
        T: Clone + PartialOrd + SampleUniform,
    {
        self.0.gen_range(range.start.clone(), range.end.clone())
    }
    /// judge an event with happenig probability 1 / p_inv happens or not
    pub fn does_happen(&mut self, p_inv: u32) -> bool {
        self.gen_range(0, p_inv) == 0
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
pub struct RandomSelecter<'a, T: PrimInt> {
    current: u64,
    offset: T,
    selected: BTreeSet<u64>,
    rng: &'a mut RngHandle,
}

impl<'a, T: PrimInt> Iterator for RandomSelecter<'a, T> {
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

impl<'a, T: PrimInt> RandomSelecter<'a, T> {
    fn reserve(self) -> ReservedSelecter<T> {
        ReservedSelecter {
            current: self.current,
            offset: self.offset,
            selected: self.selected,
        }
    }
}

pub struct ReservedSelecter<T: PrimInt> {
    current: u64,
    offset: T,
    selected: BTreeSet<u64>,
}

impl<T: PrimInt> ReservedSelecter<T> {
    pub fn into_selecter<'a>(self, rng: &'a mut RngHandle) -> RandomSelecter<'a, T> {
        RandomSelecter {
            current: self.current,
            offset: self.offset,
            selected: self.selected,
            rng: rng,
        }
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
