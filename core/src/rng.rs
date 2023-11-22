use crate::fenwick::FenwickSet;
use num_traits::PrimInt;
use rand::{
    distributions::uniform::SampleUniform, thread_rng, Error as RndError, RngCore, SeedableRng,
};
pub(crate) use rand::{seq::SliceRandom, Rng};
use rand_xorshift::XorShiftRng;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::mem;
use std::ops::{Bound, Range, RangeBounds};

fn bounds_to_range<T: PrimInt>(r: impl RangeBounds<T>) -> Range<T> {
    let s = match r.start_bound() {
        Bound::Excluded(t) => *t + T::one(),
        Bound::Included(t) => *t,
        Bound::Unbounded => T::min_value(),
    };
    let g = match r.end_bound() {
        Bound::Excluded(t) => *t,
        Bound::Included(t) => *t + T::one(),
        Bound::Unbounded => T::max_value(),
    };
    s..g
}

/// wrapper of XorShiftRng
#[derive(Clone, Serialize, Deserialize)]
pub struct RngHandle(XorShiftRng);

impl Default for RngHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub fn gen_seed() -> u128 {
    let mut rng = thread_rng();
    rng.gen()
}

pub fn gen_ranged_seed(start: u128, end: u128) -> u128 {
    let mut rng = thread_rng();
    rng.gen_range(start, end)
}

impl RngHandle {
    fn gen_seed(seed: u128) -> [u8; 16] {
        unsafe { mem::transmute::<_, [u8; 16]>(seed) }
    }
    /// create new Rng by specified seed
    pub fn from_seed(seed: u128) -> Self {
        let seed = Self::gen_seed(seed);
        RngHandle(XorShiftRng::from_seed(seed))
    }
    /// create new Rng by random seed
    pub fn new() -> Self {
        let seed: [u8; 16] = thread_rng().gen();
        RngHandle(XorShiftRng::from_seed(seed))
    }
    /// select some values randomly from given range
    pub fn select<T: PrimInt>(&mut self, range: impl RangeBounds<T>) -> RandomSelecter<T> {
        let range = bounds_to_range(range);
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
    pub fn range<T: PrimInt + SampleUniform>(&mut self, range: impl RangeBounds<T>) -> T {
        let range = bounds_to_range(range);
        let (s, e) = (range.start, range.end);
        assert!(s < e, "invalid range!!");
        self.0.gen_range(s, e)
    }
    /// judge an event with happenig probability 1 / p_inv happens or not
    pub fn does_happen(&mut self, p_inv: u32) -> bool {
        self.gen_range(0, p_inv) == 0
    }
    /// judge an event with p % chance happens or not
    pub fn parcent(&mut self, p: Parcent) -> bool {
        p.valid_check();
        self.range(1..=100) <= p.0
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Parcent(pub u32);

impl Parcent {
    fn valid_check(self) {
        debug_assert!(self.0 <= 100, "Invalid parcentage {}", self.0);
    }
    pub fn truncate(i: i64) -> Parcent {
        Parcent(cmp::min(100, cmp::max(0, i) as u32))
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
            let u = rng.select(..MAX).nth(MAX / 2).unwrap();
            bs.insert(u);
        });
    }
}
