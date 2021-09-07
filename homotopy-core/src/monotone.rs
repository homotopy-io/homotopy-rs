use std::{cmp, ops::Range};

pub type Monotone = Vec<usize>;

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
                    } else {
                        seq.push(self.constraints[i].start);
                    }
                }
                self.cur = Some(seq);
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
                } else {
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
                    } else {
                        seq.push(self.constraints[i].end - 1);
                    }
                }
                self.cur = Some(seq);
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
                } else {
                    // Decrement the first non-minimal element.
                    seq[start] -= 1;

                    // Reset all elements to the left to their maximal values while preserving monotonicity.
                    let mut max = seq[start];
                    for i in 0..start {
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
        }

        self.cur.clone()
    }
}

impl std::iter::FusedIterator for MonotoneIterator {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn monotone_sequences() {
        let iterator0_1_2 = MonotoneIterator::new(false, &[0..2, 0..2]);
        assert_eq!(iterator0_1_2.collect::<Vec<_>>(), [[0, 0], [0, 1], [1, 1]]);

        let strict_iterator0_1_2 = MonotoneIterator::new(true, &[0..2, 0..2]);
        assert_eq!(strict_iterator0_1_2.collect::<Vec<_>>(), [[0, 1]]);

        let iterator0_3_3 = MonotoneIterator::new(false, &[0..4, 0..4, 0..4]);
        assert_eq!(
            iterator0_3_3.collect::<Vec<_>>(),
            [
                [0, 0, 0],
                [0, 0, 1],
                [0, 0, 2],
                [0, 0, 3],
                [0, 1, 1],
                [0, 1, 2],
                [0, 1, 3],
                [0, 2, 2],
                [0, 2, 3],
                [0, 3, 3],
                [1, 1, 1],
                [1, 1, 2],
                [1, 1, 3],
                [1, 2, 2],
                [1, 2, 3],
                [1, 3, 3],
                [2, 2, 2],
                [2, 2, 3],
                [2, 3, 3],
                [3, 3, 3],
            ]
        );
        let strict_iterator0_3_3 = MonotoneIterator::new(true, &[0..4, 0..4, 0..4]);
        assert_eq!(
            strict_iterator0_3_3.collect::<Vec<_>>(),
            [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]]
        );

        let iterator1_3_3 = MonotoneIterator::new(false, &[1..4, 0..4, 1..4]);
        assert_eq!(
            iterator1_3_3.collect::<Vec<_>>(),
            [
                [1, 1, 1],
                [1, 1, 2],
                [1, 1, 3],
                [1, 2, 2],
                [1, 2, 3],
                [1, 3, 3],
                [2, 2, 2],
                [2, 2, 3],
                [2, 3, 3],
                [3, 3, 3],
            ]
        );
        let strict_iterator1_3_3 = MonotoneIterator::new(true, &[1..4, 0..4, 1..4]);
        assert_eq!(strict_iterator1_3_3.collect::<Vec<_>>(), [[1, 2, 3]]);

        // unsatisfiable constraints
        let invalid_ms = MonotoneIterator::new(false, &[1..2, 0..1]);
        assert!(invalid_ms.collect::<Vec<_>>().is_empty());
    }
}
