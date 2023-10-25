use std::{
    cmp,
    ops::{Index, IndexMut, Range},
    slice::SliceIndex,
};

use crate::rewrite::RewriteN;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Monotone(Vec<usize>);

impl From<Vec<usize>> for Monotone {
    fn from(v: Vec<usize>) -> Self {
        Self(v)
    }
}

impl<I: SliceIndex<[usize]>> Index<I> for Monotone {
    type Output = <I as SliceIndex<[usize]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<I: SliceIndex<[usize]>> IndexMut<I> for Monotone {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl FromIterator<usize> for Monotone {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Split {
    pub source: Range<usize>,
    pub target: usize,
}

impl Monotone {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, value: usize) {
        self.0.push(value);
    }

    pub fn slices(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().copied()
    }

    pub fn cones(&self, target_size: usize) -> impl Iterator<Item = Split> + '_ {
        let mut prev = 0;
        (0..target_size).map(move |target| {
            let source = self.preimage_from(prev, target);
            prev = source.end;
            Split { source, target }
        })
    }

    /// Compose two monotones maps.
    pub fn compose(&self, other: &Self) -> Option<Self> {
        let mut seq = Vec::with_capacity(self.len());
        for i in self.slices() {
            if i >= other.len() {
                return None;
            }
            seq.push(other[i]);
        }
        Some(seq.into())
    }

    /// Given a monotone map f : [n] -> [m] in Δ, compute its dual f' : [m + 1] -> [n + 1] in Δ=.
    #[must_use]
    pub fn dual(&self, target_size: usize) -> Self {
        let mut seq = Vec::with_capacity(target_size + 1);
        for i in 0..target_size + 1 {
            // Convert i ∈ [m + 1] into a monotone g : [m] -> [2].
            let g = [[0].repeat(i), [1].repeat(target_size - i)].concat().into();

            // Compute the composite g∘f : [n] -> [2].
            let comp = self.compose(&g).unwrap();

            // Convert g∘f : [n] -> [2] back into an element f'(i) ∈ [n + 1].
            seq.push(comp.slices().position(|j| j == 1).unwrap_or(self.len()));
        }
        seq.into()
    }

    /// Given a monotone map f : [m + 1] -> [n + 1] in Δ=, compute its inverse dual f* : [n] -> [m] in Δ.
    #[must_use]
    pub fn dual_inv(&self, target_size: usize) -> Self {
        // Check that f preserves top and bottom elements.
        assert_eq!(self[0], 0);
        assert_eq!(self[self.len() - 1], target_size - 1);

        // Forget f is in Δ= and compute the dual f' : [n + 2] -> [m + 2].
        let dual = self.dual(target_size);

        // Strip away the first and last elements to get the inverse dual f* : [n] -> [m].
        dual[1..target_size].iter().map(|&i| i - 1).collect()
    }

    pub fn preimage(&self, target_index: usize) -> Range<usize> {
        self.preimage_from(0, target_index)
    }

    #[inline]
    fn preimage_from(&self, begin: usize, target_index: usize) -> Range<usize> {
        let source_size = self.len();
        for start in begin..source_size {
            if self[start] >= target_index {
                let mut end = start;
                while end < source_size && self[end] == target_index {
                    end += 1;
                }
                return start..end;
            }
        }
        source_size..source_size
    }
}

impl RewriteN {
    pub fn singular_monotone(&self, source_size: usize) -> Monotone {
        (0..source_size).map(|i| self.singular_image(i)).collect()
    }

    pub fn regular_monotone(&self, target_size: usize) -> Monotone {
        (0..target_size + 1)
            .map(|i| self.regular_image(i))
            .collect()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct MonotoneIterator {
    cur: Option<Monotone>,
    pub strict: bool, // whether the sequences should be *strictly* monotone.
    pub constraints: Vec<Range<usize>>, // element-wise range constraints for the sequences.
}

#[allow(clippy::len_without_is_empty)]
impl MonotoneIterator {
    pub fn new(strict: bool, constraints: &[Range<usize>]) -> Self {
        let len = constraints.len();

        // We need to tighten the constraints as the iterator assumes the constraints are tight.
        let mut tight_constraints = constraints.to_owned();
        if len > 1 {
            let mut min = tight_constraints[0].start;
            let mut max = tight_constraints[len - 1].end;
            for i in 1..len {
                if strict {
                    min = cmp::max(min + 1, tight_constraints[i].start);
                    max = cmp::min(max - 1, tight_constraints[len - i - 1].end);
                } else {
                    min = cmp::max(min, tight_constraints[i].start);
                    max = cmp::min(max, tight_constraints[len - i - 1].end);
                }
                tight_constraints[i].start = min;
                tight_constraints[len - i - 1].end = max;
            }
        }

        Self {
            cur: None,
            strict,
            constraints: tight_constraints,
        }
    }

    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    /// Checks if the constraints can only be satisfied by the trivial sequence.
    pub fn is_trivial(&self, target_size: usize) -> bool {
        self.constraints.len() == target_size
            && self
                .constraints
                .iter()
                .enumerate()
                .all(|(i, range)| range.start == i && range.end == i + 1)
    }

    /// Restrict the iterator to only those sequences generated by another iterator.
    pub fn restrict_to(&mut self, other: &Self) {
        assert_eq!(self.len(), other.len());
        for i in 0..self.len() {
            self.constraints[i].start =
                cmp::max(self.constraints[i].start, other.constraints[i].start);
            self.constraints[i].end = cmp::min(self.constraints[i].end, other.constraints[i].end);
        }
    }
}

#[allow(clippy::needless_range_loop)]
impl Iterator for MonotoneIterator {
    type Item = Monotone;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.cur {
            None => {
                // Set all elements to their minimal values.
                let mut seq = Vec::with_capacity(self.len());
                for i in 0..self.len() {
                    // If the constraint is invalid, the iterator is empty.
                    if self.constraints[i].is_empty() {
                        return None;
                    }
                    seq.push(self.constraints[i].start);
                }
                self.cur = Some(seq.into());
            }
            Some(seq) => {
                // Find the last non-maximal element.
                let mut end = seq.len();
                while end > 0 && seq[end - 1] == self.constraints[end - 1].end - 1 {
                    end -= 1;
                }

                if end == 0 {
                    // All elements are maximal, so we have reached the end.
                    return None;
                }

                // Increment the last non-maximal element.
                seq[end - 1] += 1;

                // Reset all elements to the right to their minimal values while preserving monotonicity.
                let mut min = seq[end - 1];
                for i in end..seq.len() {
                    if self.strict {
                        min = cmp::max(min + 1, self.constraints[i].start);
                    } else {
                        min = cmp::max(min, self.constraints[i].start);
                    }
                    assert!(min < self.constraints[i].end);
                    seq[i] = min;
                }
            }
        }

        self.cur.clone()
    }
}

#[allow(clippy::needless_range_loop)]
impl DoubleEndedIterator for MonotoneIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.cur {
            None => {
                // Set all elements to their maximal values.
                let mut seq = Vec::with_capacity(self.len());
                for i in 0..self.len() {
                    // If the constraint is invalid, the iterator is empty.
                    if self.constraints[i].is_empty() {
                        return None;
                    }
                    seq.push(self.constraints[i].end - 1);
                }
                self.cur = Some(seq.into());
            }
            Some(seq) => {
                // Find the first non-minimal element.
                let mut start = 0;
                while start < seq.len() && seq[start] == self.constraints[start].start {
                    start += 1;
                }

                if start == seq.len() {
                    // All elements are maximal, we have reached the end.
                    return None;
                }

                // Decrement the first non-minimal element.
                seq[start] -= 1;

                // Reset all elements to the left to their maximal values while preserving monotonicity.
                let mut max = seq[start];
                for i in (0..start).rev() {
                    if self.strict {
                        max = cmp::min(max - 1, self.constraints[i].end - 1);
                    } else {
                        max = cmp::min(max, self.constraints[i].end - 1);
                    }
                    assert!(max >= self.constraints[i].start);
                    seq[i] = max;
                }
            }
        }

        self.cur.clone()
    }
}

impl std::iter::FusedIterator for MonotoneIterator {}
