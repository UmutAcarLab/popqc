extern crate alloc;
use alloc::vec::Vec;
use dashmap::DashMap;
use rayon::prelude::*;
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct FenwickTree {
    pub inner: Vec<usize>,
}

impl FromIterator<usize> for FenwickTree {
    /// Creates a new fenwick tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use ftree::FenwickTree;
    ///
    /// let lengths: [usize; 5] = [1, 6, 3, 9, 2];
    /// // This is how lengths fenwick tree will look like internally
    /// let fenwick_tree: Vec<usize> = vec![1, 7, 3, 19, 2];
    /// // And this is how it can be constructed
    /// let initialized_fenwick_tree = FenwickTree::from_iter(lengths);
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = usize>,
    {
        let mut inner: Vec<usize> = iter.into_iter().collect();
        let n = inner.len();

        for i in 0..n {
            let parent = i | (i + 1);
            if parent < n {
                let child = inner[i];
                inner[parent] += child;
            }
        }

        FenwickTree { inner }
    }
}

impl<const N: usize> From<[usize; N]> for FenwickTree {
    fn from(value: [usize; N]) -> Self {
        FenwickTree::from_iter(value)
    }
}
#[allow(clippy::new_without_default)]
impl FenwickTree {
    /// Creates an empty fenwick tree.
    ///
    pub const fn new() -> Self {
        let inner = Vec::new();

        Self { inner }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl FenwickTree {
    /// Computes the prefix sum up until the desired index.
    ///
    /// The prefix sum up until the zeroth element is 0, since there is nothing before it.
    ///
    /// The prefix sum up until an index larger than the length is undefined, since every
    /// element after the length - 1 is undefined, therefore it will panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use ftree::FenwickTree;
    ///
    /// let lengths = [1, 6, 3, 9, 2];
    /// let fenwick_array = FenwickTree::from_iter(lengths);
    ///
    /// let cases: Vec<(usize, usize)> =
    ///  vec![(0, 0), (1, 1), (2, 7), (3, 10), (4, 19), (5, 21)];
    ///
    /// cases
    ///   .into_iter()
    ///   .for_each(|(idx, expected_sum)| assert_eq!(fenwick_array.prefix_sum(idx, 0), expected_sum))
    /// ```
    pub fn prefix_sum(&self, index: usize, mut sum: usize) -> usize {
        {
            assert!(index < self.inner.len() + 1);

            let mut current_idx = index;

            while current_idx > 0 {
                sum += self.inner[current_idx - 1];
                current_idx &= current_idx - 1
            }

            sum
        }
    }
    /// Increments a given index with a difference.
    ///
    /// # Examples
    ///
    /// ```
    /// use ftree::FenwickTree;
    ///
    /// let lengths = [1, 6, 3, 9, 2];
    /// let mut fenwick_array = FenwickTree::from_iter(lengths);
    ///
    /// let cases: Vec<(usize, usize)> = vec![(0, 0), (1, 2), (2, 8), (3, 11), (4, 20), (5, 22)];
    ///
    /// fenwick_array.add_at(0, 1);
    ///
    /// cases
    ///   .into_iter()
    ///   .for_each(|(idx, expected_sum)| assert_eq!(fenwick_array.prefix_sum(idx, 0), expected_sum))
    /// ```
    ///
    pub fn collect_same_key(&mut self, index: Vec<(usize, isize)>) -> Vec<(usize, isize)> {
        let merged = DashMap::new();
        index.par_iter().for_each(|(key, value)| {
            *merged.entry(*key).or_insert(0) += value;
        });
        let result: Vec<(usize, isize)> = merged.into_iter().collect();
        result
    }

    fn update_batch(&mut self, index: &[(usize, isize)]) {
        if index.iter().any(|&(i, _)| i >= self.inner.len()) {
            panic!("Index out of bounds");
        }

        let ptr_as_usize = self.inner.as_mut_ptr() as usize;

        index.par_iter().for_each(|&(i, v)| {
            let ptr = ptr_as_usize as *mut usize;
            unsafe {
                if v >= 0 {
                    *ptr.add(i) += v as usize;
                } else {
                    *ptr.add(i) -= (-v) as usize;
                }
            }
        });
    }

    pub fn add_at_batch(&mut self, index: Vec<(usize, isize)>) {
        let mut new_index = index;
        while !new_index.is_empty() {
            self.update_batch(&new_index);
            new_index = new_index
                .par_iter()
                .map(|(key, value)| ((key + 1) | key, *value))
                .filter(|(i, _)| self.inner.get(*i).is_some())
                .collect();
            new_index = self.collect_same_key(new_index);
        }
    }

    pub fn add_at(&mut self, index: usize, diff: isize) {
        let mut current_idx = index;

        while let Some(value) = self.inner.get_mut(current_idx) {
            if diff >= 0 {
                *value += diff as usize;
            } else {
                *value -= (-diff) as usize;
            }
            current_idx |= current_idx + 1;
        }
    }
    /// Appends a new value to the end of the Fenwick tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use ftree::FenwickTree;
    ///
    /// let mut fenwick_array = FenwickTree::from_iter([1, 6, 3].into_iter());
    /// fenwick_array.push(9);
    ///
    /// // Check prefix sums after pushing
    /// assert_eq!(fenwick_array.prefix_sum(1, 0), 1);  // sum of [1]
    /// assert_eq!(fenwick_array.prefix_sum(2, 0), 7);  // sum of [1, 6]
    /// assert_eq!(fenwick_array.prefix_sum(3, 0), 10); // sum of [1, 6, 3]
    /// assert_eq!(fenwick_array.prefix_sum(4, 0), 19); // sum of [1, 6, 3, 9]
    /// ```
    pub fn push(&mut self, value: usize) {
        let index = self.inner.len();
        self.inner.push(value);

        for i in 0..index {
            let parent = i | (i + 1);
            let parent_val = self.inner[i];
            if parent == index {
                self.inner[index] += parent_val;
            }
        }
    }
    /// Subtracts a difference from a given index.
    ///
    /// # Examples
    ///
    /// ```
    /// use ftree::FenwickTree;
    ///
    /// let lengths = [1, 6, 3, 9, 2];
    /// let mut fenwick_array = FenwickTree::from_iter(lengths);
    ///
    /// let cases: Vec<(usize, usize)> = vec![(0, 0), (1, 0), (2, 6), (3, 9), (4, 18), (5, 20)];
    ///
    /// fenwick_array.sub_at(0, 1);
    ///
    /// cases
    ///   .into_iter()
    ///   .for_each(|(idx, expected_sum)| assert_eq!(fenwick_array.prefix_sum(idx, 0), expected_sum))
    /// ```
    pub fn sub_at(&mut self, index: usize, diff: usize) {
        let mut current_idx = index;

        while let Some(value) = self.inner.get_mut(current_idx) {
            *value -= diff;
            current_idx |= current_idx + 1;
        }
    }
    /// Removes the last element from the Fenwick tree.
    ///
    /// Returns `false` if the tree is empty, and true otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use ftree::FenwickTree;
    ///
    /// let mut fenwick_array = FenwickTree::from_iter([1, 6, 3, 9].into_iter());
    ///
    /// assert_eq!(fenwick_array.pop(), true);  
    /// assert_eq!(fenwick_array.prefix_sum(3, 0), 10);  // sum of remaining [1, 6, 3]
    ///
    /// // Can continue popping
    /// assert_eq!(fenwick_array.pop(), true);
    /// assert_eq!(fenwick_array.prefix_sum(2, 0), 7);   // sum of remaining [1, 6]
    ///
    /// // Returns false when empty
    /// fenwick_array.pop();  // removes 6
    /// fenwick_array.pop();  // removes 1
    /// assert_eq!(fenwick_array.pop(), false);
    /// ```
    pub fn pop(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        let last_idx = self.inner.len() - 1;

        let sum_excl = self.prefix_sum(last_idx, 0);
        let sum_incl = self.prefix_sum(last_idx + 1, 0);
        let mut reconstructed_value = sum_incl;
        reconstructed_value -= sum_excl;

        self.sub_at(last_idx, reconstructed_value);

        self.inner.pop().is_some()
    }
    /// Given a sum, finds the slot in which in which it would be "contained" within the original
    /// array.
    ///
    /// # Examples
    ///
    /// ```
    /// use ftree::FenwickTree;
    ///
    /// let lengths = [1, 6, 3, 9, 2];
    /// let mut fenwick_array = FenwickTree::from_iter(lengths);
    ///
    /// let cases: Vec<(usize, usize)> = vec![(0, 0), (6, 1), (9, 2), (18, 3), (20, 4)];
    ///
    /// cases
    ///   .into_iter()
    ///   .for_each(|(prefix_sum, idx)| assert_eq!(fenwick_array.index_of(prefix_sum), idx))
    /// ```
    pub fn index_of(&self, mut prefix_sum: usize) -> usize {
        let mut index = 0;
        let mut probe = most_significant_bit(self.inner.len()) * 2;

        while probe > 0 {
            let lsb = least_significant_bit(probe);
            let half_lsb = lsb / 2;
            let other_half_lsb = lsb - half_lsb;

            if let Some(value) = self.inner.get(probe - 1) {
                if *value < prefix_sum {
                    index = probe;
                    prefix_sum -= *value;

                    probe += half_lsb;

                    if half_lsb > 0 {
                        continue;
                    }
                }
            }

            if lsb % 2 > 0 {
                break;
            }

            probe -= other_half_lsb;
        }

        index
    }
}

const fn least_significant_bit(n: usize) -> usize {
    n & n.wrapping_neg()
}

const fn most_significant_bit(n: usize) -> usize {
    if n == 0 {
        0
    } else {
        1 << (usize::BITS - 1 - n.leading_zeros())
    }
}

#[cfg(test)]
mod tests {
    use super::FenwickTree;

    #[test]
    fn test_new() {
        let lengths: [usize; 5] = [1, 6, 3, 9, 2];
        let expected_index: Vec<usize> = vec![1, 7, 3, 19, 2];
        let actual_index = FenwickTree::from_iter(lengths);
        assert_eq!(expected_index, actual_index.inner)
    }

    #[test]
    fn test_prefix_sum() {
        let lengths = [1, 6, 3, 9, 2];
        let fenwick_array = FenwickTree::from_iter(lengths);

        let cases: Vec<(usize, usize)> = vec![(0, 0), (1, 1), (2, 7), (3, 10), (4, 19), (5, 21)];
        // The prefix sum up until the zeroth element is 0, since there is nothing before it
        // The prefix sum up until an index larger than the length is undefined, since every
        // element after the length - 1 is undefined
        cases.into_iter().for_each(|(idx, expected_sum)| {
            assert_eq!(fenwick_array.prefix_sum(idx, 0), expected_sum)
        })
    }

    #[test]
    fn test_par_update() {
        let lengths = [1; 5000];
        let cases: Vec<(usize, isize)> = vec![
            (0, -1),
            (1, -1),
            (2, -1),
            (3, -1),
            (4, -1),
            (5, -1),
            (6, 1),
            (4999, -1),
        ];

        let mut fenwick_array = FenwickTree::from_iter(lengths);
        fenwick_array.add_at_batch(cases.clone());
        let inner1 = fenwick_array.inner.clone();
        let mut fenwick_array2 = FenwickTree::from_iter(lengths);
        cases
            .iter()
            .for_each(|(i, v)| fenwick_array2.add_at(*i, *v));
        let inner2 = fenwick_array2.inner.clone();
        assert_eq!(inner1, inner2);
        // The prefix sum up until the zeroth element is 0, since there is nothing before it
        // The prefix sum up until an index larger than the length is undefined, since every
        // element after the length - 1 is undefined
    }

    #[test]
    fn test_update_index() {
        let lengths = [1, 6, 3, 9, 2];
        let mut fenwick_array = FenwickTree::from_iter(lengths);

        let cases: Vec<(usize, usize)> = vec![(0, 2), (1, 8), (2, 3), (3, 20), (4, 2)];

        fenwick_array.add_at(0, 1);

        cases
            .into_iter()
            .for_each(|(idx, expected_value)| assert_eq!(fenwick_array.inner[idx], expected_value))
    }

    #[test]
    fn test_index_of() {
        let lengths = [1, 6, 3, 9, 2];
        let fenwick_array = FenwickTree::from_iter(lengths);

        let cases: Vec<(usize, usize)> = vec![(0, 0), (6, 1), (9, 2), (18, 3), (20, 4)];

        cases
            .into_iter()
            .for_each(|(prefix_sum, idx)| assert_eq!(fenwick_array.index_of(prefix_sum), idx))
    }

    // #[test]
    // #[ntest::timeout(1000)]
    // fn test_zero_array() {
    //     // test for a regression where index_of in an array containing only 0 would loop endlessly
    //     let f0: FenwickTree<usize> = FenwickTree::from([0]);
    //     assert_eq!(f0.prefix_sum(0, 0), 0);
    //     assert_eq!(f0.index_of(1), 1);
    // }

    #[test]
    fn test_push_empty() {
        let mut fenwick = FenwickTree::new();
        fenwick.push(5);
        assert_eq!(fenwick.inner, vec![5]);
        assert_eq!(fenwick.prefix_sum(1, 0), 5);
    }

    #[test]
    fn test_push_sequence() {
        let mut fenwick = FenwickTree::new();
        let values = [1, 6, 3, 9, 2];
        let expected_sums = vec![(1, 1), (2, 7), (3, 10), (4, 19), (5, 21)];

        for &v in values.iter() {
            fenwick.push(v);
        }

        expected_sums
            .into_iter()
            .for_each(|(idx, expected_sum)| assert_eq!(fenwick.prefix_sum(idx, 0), expected_sum));
    }

    #[test]
    fn test_push_after_initialization() {
        let mut fenwick = FenwickTree::from_iter([1, 6, 3].into_iter());
        fenwick.push(9);
        fenwick.push(2);

        let expected_sums = vec![(1, 1), (2, 7), (3, 10), (4, 19), (5, 21)];
        expected_sums
            .into_iter()
            .for_each(|(idx, expected_sum)| assert_eq!(fenwick.prefix_sum(idx, 0), expected_sum));
    }

    #[test]
    fn test_pop_empty() {
        let mut fenwick: FenwickTree = FenwickTree::new();
        assert_eq!(fenwick.pop(), false);
    }

    #[test]
    fn test_pop_single() {
        let mut fenwick = FenwickTree::from_iter([5].into_iter());
        assert_eq!(fenwick.pop(), true);
        assert!(fenwick.is_empty());
    }

    #[test]
    fn test_pop_sequence() {
        let mut fenwick = FenwickTree::from_iter([1, 6, 3, 9, 2].into_iter());
        assert_eq!(fenwick.pop(), true);
        assert_eq!(fenwick.pop(), true);
        assert_eq!(fenwick.pop(), true);

        assert_eq!(fenwick.prefix_sum(1, 0), 1);
        assert_eq!(fenwick.prefix_sum(2, 0), 7);
    }

    #[test]
    fn test_push_pop_alternating() {
        let mut fenwick = FenwickTree::new();

        fenwick.push(1);
        fenwick.push(6);
        assert_eq!(fenwick.pop(), true);
        fenwick.push(3);
        assert_eq!(fenwick.pop(), true);
        fenwick.push(9);
        fenwick.push(2);
        assert_eq!(fenwick.pop(), true);

        assert_eq!(fenwick.prefix_sum(1, 0), 1);
        assert_eq!(fenwick.prefix_sum(2, 0), 10);
    }

    #[test]
    fn test_zero_handling() {
        let mut fenwick = FenwickTree::new();
        fenwick.push(0);
        fenwick.push(0);
        assert_eq!(fenwick.pop(), true);
        assert_eq!(fenwick.prefix_sum(1, 0), 0);
    }

    // #[test]
    // fn test_negative_values() {
    //     let mut fenwick: FenwickTree<i32> = FenwickTree::new();
    //     fenwick.push(-1);
    //     fenwick.push(2);
    //     fenwick.push(-3);

    //     assert_eq!(fenwick.pop(), true);
    //     assert_eq!(fenwick.prefix_sum(2, 0), 1);
    // }
}
