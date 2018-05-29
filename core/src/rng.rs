use fenwick::FenwickSet;
use num_traits::PrimInt;
pub(crate) use rand::Rng;
use rand::{
    distributions::uniform::SampleUniform, thread_rng, Error as RndError, RngCore, SeedableRng,
    XorShiftRng,
};
#[cfg(test)]
use std::convert;
use std::fmt::Debug;
use std::ops::Range;
/// wrapper of XorShiftRng
#[derive(Clone, Serialize, Deserialize)]
pub struct RngHandle(XorShiftRng);

impl Default for RngHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub fn gen_seed() -> u64 {
    let mut rng = thread_rng();
    rng.gen()
}

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
        let width = width.to_usize().expect("[RngHandle::select] NumCast error");
        if width > 10_000_000 {
            panic!("[RngHandle::select] too large range");
        }
        RandomSelecter {
            offset: range.start,
            selected: FenwickSet::from_range(0..width),
            rng: self,
        }
    }
    /// select some values randomly using given FenwickSet
    pub fn select_with<T: PrimInt>(&mut self, set: FenwickSet) -> RandomSelecter<T> {
        RandomSelecter {
            offset: T::zero(),
            selected: set,
            rng: self,
        }
    }
    /// wrapper of gen_range which takes Range
    pub fn range<T>(&mut self, range: Range<T>) -> T
    where
        T: Clone + PartialOrd + SampleUniform + Debug,
    {
        let (s, e) = (range.start.clone(), range.end.clone());
        assert!(s < e, "invalid range {:?}..{:?}", s, e);
        self.0.gen_range(s, e)
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
    offset: T,
    selected: FenwickSet,
    rng: &'a mut RngHandle,
}

impl<'a, T: PrimInt> Iterator for RandomSelecter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let num_rests = self.selected.len();
        if num_rests == 0 {
            return None;
        }
        let n = self.rng.gen_range(0, num_rests);
        let res = self
            .selected
            .nth(n)
            .expect("[RandomSelecter::next] no nth element(maybe logic bug)");
        self.selected.remove(res);
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

}

#[cfg(test)]
mod selecter_test {
    use super::*;
    use fixedbitset::FixedBitSet;
    #[test]
    fn rng_select_normal() {
        let mut rng = RngHandle::new();
        let max = 100;
        let bs: FixedBitSet = rng.select(0..max).take(max).collect();
        (0..max).for_each(|i| {
            assert!(bs.contains(i));
        });
    }
    #[test]
    fn rng_select_advanced() {
        let mut rng = RngHandle::new();
        let (l, r) = (300, 2400);
        let bs: FixedBitSet = rng.select(l..r).take(1000).collect();
        let mut bs2 = FixedBitSet::with_capacity(5000);
        bs.ones().for_each(|u| {
            assert!(!bs2.contains(u));
            bs2.insert(u);
            assert!(l <= u && u < r);
        });
    }
}

#[cfg(feature = "bench")]
mod selecter_bench {
    use super::*;
    use fixedbitset::FixedBitSet;
    use test::Bencher;
    #[bench]
    fn select_bench(b: &mut Bencher) {
        const MAX: usize = 1_000_000;
        let mut bs = FixedBitSet::from_capacity(MAX);
        b.iter(|| {
            let mut rng = RngHandle::new();
            let u = rng.select(0..MAX).nth(MAX / 2).unwrap();
            bs.insert(u);
        });
    }
}