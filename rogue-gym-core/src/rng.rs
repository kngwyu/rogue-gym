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
        let width = width.to_usize().expect("[RngHandle::select] NumCast error");
        if width > 10_000_000 {
            panic!("[RngHandle::select] too large range");
        }
        let mut fenwick = FenwickTree::new(width);
        (0..width).for_each(|i| fenwick.add(i, 1));
        RandomSelecter {
            current: width,
            offset: range.start,
            selected: fenwick,
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
    current: usize,
    offset: T,
    selected: FenwickTree,
    rng: &'a mut RngHandle,
}

impl<'a, T: PrimInt> Iterator for RandomSelecter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.current == 0 {
            return None;
        }
        let r = self.rng.gen_range(0, self.current) + 1;
        let res = self.selected.lower_bound(r as i64);
        self.selected.add(res, -1);
        self.current -= 1;
        let res = T::from(res).expect("[RngSelect::Iterator::next] NumCast error") + self.offset;
        Some(res)
    }
}

impl<'a, T: PrimInt> RandomSelecter<'a, T> {
    pub fn reserve(self) -> ReservedSelecter<T> {
        ReservedSelecter {
            current: self.current,
            offset: self.offset,
            selected: self.selected,
        }
    }
}

pub struct ReservedSelecter<T: PrimInt> {
    current: usize,
    offset: T,
    selected: FenwickTree,
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
    pub fn select_once(&mut self, rng: &mut RngHandle) -> Option<T> {
        if self.current == 0 {
            return None;
        }
        let r = rng.gen_range(0, self.current) + 1;
        let res = self.selected.lower_bound(r as i64);
        self.selected.add(res, -1);
        self.current -= 1;
        let res = T::from(res).expect("[RngSelect::Iterator::next] NumCast error") + self.offset;
        Some(res)
    }
}

/// simple 0-indexed fenwick tree
struct FenwickTree {
    inner: Vec<i64>,
    len: isize,
}

impl FenwickTree {
    fn new(length: usize) -> Self {
        FenwickTree {
            inner: vec![0; length + 1],
            len: length as isize,
        }
    }
    /// add plus to array[idx]
    fn add(&mut self, idx: usize, plus: i64) {
        let mut idx = (idx + 1) as isize;
        while idx <= self.len {
            self.inner[idx as usize] += plus;
            idx += idx & -idx;
        }
    }
    /// return sum of range 0..range_max
    fn sum(&self, range_max: usize) -> i64 {
        let mut sum = 0;
        let mut idx = range_max as isize;
        while idx > 0 {
            sum += self.inner[idx as usize];
            idx -= idx & -idx;
        }
        sum
    }
    /// return sum of range 0..range_max
    fn sum_range(&self, range: Range<usize>) -> i64 {
        let sum1 = self.sum(range.end);
        if range.start == 0 {
            return sum1;
        } else {
            let sum2 = self.sum(range.start - 1);
            sum1 - sum2
        }
    }
    /// return minimum i where array[0] + array[1] + ... + array[i] >= query (1 <= i <= N)
    fn lower_bound(&self, mut query: i64) -> usize {
        if query <= 0 {
            return 0;
        }
        let mut k = 1;
        while k <= self.len {
            k *= 2;
        }
        let mut cur = 0;
        while k > 0 {
            k /= 2;
            let nxt = cur + k;
            if nxt > self.len {
                continue;
            }
            let val = self.inner[nxt as usize];
            if val < query {
                query -= val;
                cur += k;
            }
        }
        cur as usize
    }
}

#[cfg(test)]
mod fenwick_test {
    use super::*;
    #[test]
    fn sum() {
        let max = 10000;
        let mut fenwick = FenwickTree::new(max);
        let range = 0..max; // 3400..7000;
        let mut rng = RngHandle::new();
        let mut sum = 0;
        for _ in 0..1 {
            let plus = rng.gen_range(0, 1000000000i64);
            let id = rng.gen_range(0, max);
            fenwick.add(id, plus);
            if range.start <= id && id < range.end {
                sum += plus;
            }
        }
        assert_eq!(sum, fenwick.sum_range(range));
    }
    #[test]
    fn lower_bound() {
        let max = 100;
        let mut fenwick = FenwickTree::new(max);
        for x in 0..max {
            fenwick.add(x, x as i64);
        }
        let mut sum = 0;
        for x in 0..max {
            sum += x as i64;
            assert_eq!(fenwick.lower_bound(sum), x);
        }
    }
}

#[cfg(test)]
mod rng_test {
    use super::*;
    use fixedbitset::FixedBitSet;
    use test::Bencher;
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
        let max = 100;
        let bs: FixedBitSet = rng.select(0..max).take(max).collect();
        (0..max).for_each(|i| {
            assert!(bs.contains(i));
        });
    }
    #[test]
    fn rng_select2() {
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
    #[bench]
    fn select_bench(b: &mut Bencher) {
        const MAX: usize = 1_000_000;
        let mut bs = FixedBitSet::with_capacity(MAX);
        b.iter(|| {
            let mut rng = RngHandle::new();
            let u = rng.select(0..MAX).nth(MAX / 2).unwrap();
            bs.insert(u);
        });
    }
}
