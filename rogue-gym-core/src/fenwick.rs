use rng::{Rng, RngHandle};
use std::ops::Range;

/// a set implementation using Fenwick Tree
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FenwickSet {
    inner: FenwickTree,
    num_elements: usize,
    max_val: usize,
}

impl FenwickSet {
    /// create a new set with capacity [0..n)
    pub fn with_capacity(n: usize) -> Self {
        assert!(
            n <= 5_000_000_0,
            "We can't construct too big FenwickSet: size {}",
            n
        );
        FenwickSet {
            inner: FenwickTree::new(n),
            num_elements: 0,
            max_val: n - 1,
        }
    }
    /// create a new set from range `r` with the capacity [0..r.end)
    pub fn from_range(range: Range<usize>) -> Self {
        let (start, end) = (range.start, range.end);
        let mut set = FenwickSet::with_capacity(end);
        (start..end).for_each(|i| {
            set.insert(i);
        });
        set
    }
    /// Insert an element `elem` into set
    /// if `elem` is already in the set, return false.
    /// if not, return true.
    pub fn insert(&mut self, elem: usize) -> bool {
        if elem > self.max_val || self.contains(elem) {
            false
        } else {
            self.inner.add(elem, 1);
            self.num_elements += 1;
            true
        }
    }
    /// Remove an element `elem` from set
    /// if `elem` is in the set, return true.
    /// if not, return false.
    pub fn remove(&mut self, elem: usize) -> bool {
        if elem > self.max_val || !self.contains(elem) {
            false
        } else {
            self.inner.add(elem, -1);
            self.num_elements -= 1;
            true
        }
    }
    /// Checks if the set cotains a element `elem`
    pub fn contains(&self, elem: usize) -> bool {
        if elem > self.max_val {
            return false;
        }
        self.inner.sum_range(elem..elem + 1) == 1
    }
    /// return nth-smallest element in the set
    pub fn nth(&self, n: usize) -> Option<usize> {
        let res = self.inner.lower_bound(n as i64 + 1);
        if res > self.max_val {
            None
        } else {
            Some(res)
        }
    }
    /// return how many elements the set has
    pub fn len(&self) -> usize {
        self.num_elements
    }
    /// select one integer randomly from the set
    pub fn select<R: Rng>(&self, rng: &mut R) -> Option<usize> {
        if self.num_elements == 0 {
            return None;
        }
        let num = rng.gen_range(0, self.num_elements);
        self.nth(num)
    }
}

impl IntoIterator for FenwickSet {
    type Item = usize;
    type IntoIter = FenwickSetIter;
    fn into_iter(self) -> Self::IntoIter {
        FenwickSetIter {
            fwt: self.inner,
            current: 0,
            before: 0,
        }
    }
}

/// Iterator for FenwickSet
pub struct FenwickSetIter {
    fwt: FenwickTree,
    current: isize,
    before: i64,
}

impl Iterator for FenwickSetIter {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        while self.current < self.fwt.len {
            self.current += 1;
            let sum = self.fwt.sum(self.current as usize);
            let diff = sum - self.before;
            self.before = sum;
            if diff == 1 {
                return Some(self.current as usize - 1);
            }
        }
        None
    }
}

/// simple 0-indexed fenwick tree
#[derive(Clone, Debug, Serialize, Deserialize)]
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
            let sum2 = self.sum(range.start);
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
mod fenwick_set_test {
    use super::*;
    use std::collections::{BTreeSet, HashSet};
    use std::iter::FromIterator;
    #[test]
    fn same_as_hashset() {
        let mut rng = RngHandle::new();
        let max = 1_000_000;
        let mut fws = FenwickSet::with_capacity(max);
        let mut hash = HashSet::new();
        for _ in 0..100000 {
            let num = rng.range(0..max);
            assert_eq!(fws.insert(num), hash.insert(num));
        }
        for _ in 0..5000 {
            let num = rng.range(0..max);
            assert_eq!(fws.remove(num), hash.remove(&num));
        }
        let hash_from_fws: HashSet<usize> = HashSet::from_iter(fws);
        assert_eq!(hash, hash_from_fws);
    }
    #[test]
    fn iter() {
        let max = 1_000_000;
        let mut fws = FenwickSet::with_capacity(max);
        let mut rng = RngHandle::new();
        let mut bts = BTreeSet::new();
        for _ in 0..1000 {
            let num = rng.range(0..max);
            bts.insert(num);
            fws.insert(num);
        }
        assert!(bts.into_iter().zip(fws.into_iter()).all(|(a, b)| a == b));
    }
    #[test]
    fn nth() {
        let max = 1_000_000;
        let mut fws = FenwickSet::with_capacity(max);
        for i in (0..1000).filter(|&i| i % 2 == 1) {
            fws.insert(i);
        }
        assert_eq!(fws.nth(9), Some(19));
        assert_eq!(fws.nth(499), Some(999));
        assert_eq!(fws.nth(500), None);
    }
    #[test]
    fn select() {
        let max = 1_000_000;
        let mut fws = FenwickSet::with_capacity(max);
        let mut rng = RngHandle::new();
        assert_eq!(fws.select(&mut rng), None);
        for _ in 0..1000 {
            let num = rng.range(0..max);
            fws.insert(num);
        }
        for _ in 0..10 {
            let num = fws.select(&mut rng).unwrap();
            assert!(fws.contains(num));
        }
    }
    #[test]
    fn invalid_value() {
        let max = 1_000_000;
        let mut fws = FenwickSet::with_capacity(max);
        for i in max..max + 10 {
            assert!(!fws.insert(i));
            assert!(!fws.remove(i));
        }
    }
    #[test]
    fn from_range() {
        let (start, end) = (40, 500);
        let mut fws = FenwickSet::from_range(start..end);
        for i in 0..1000 {
            let in_range = start <= i && i < end;
            assert_eq!(fws.contains(i), in_range);
        }
    }
}

#[cfg(test)]
mod fenwick_tree_test {
    use super::*;
    #[test]
    fn sum() {
        let max = 10000;
        let mut fenwick = FenwickTree::new(max);
        let range = 0..max; // 3400..7000;
        let mut rng = RngHandle::new();
        let mut sum = 0;
        for _ in 0..1 {
            let plus = rng.range(0..1000000000i64);
            let id = rng.range(0..max);
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
        assert_eq!(fenwick.lower_bound(sum + 10), max);
    }
}
