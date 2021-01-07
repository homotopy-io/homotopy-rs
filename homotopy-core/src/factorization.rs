pub struct MonotoneSequences {
    cur: Option<Vec<usize>>,

    // invariant: ∀ x ∈ cur(end, len). x = max
    end: usize,

    pub len: usize,
    pub max: usize,
}

impl MonotoneSequences {
    pub fn new(len: usize, max: usize) -> MonotoneSequences {
        MonotoneSequences {
            cur: None,
            end: len - 1,
            len,
            max,
        }
    }
}

impl Iterator for MonotoneSequences {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.cur {
            None => {
                // first monotone sequence is all 0s
                self.cur = Some([0].repeat(self.len));
            }
            Some(seq) => {
                if seq != &[self.max].repeat(self.len) {
                    seq[self.end] += 1; // increment last non-max digit
                    if seq[self.end] == self.max {
                        self.end = self.end.saturating_sub(1) // maintain invariant
                    } else {
                        for i in (self.end + 1)..self.len {
                            // set all values to the right to it
                            seq[i] = seq[self.end]
                        }
                        self.end = self.len - 1 // maintain invariant
                    }
                } else {
                    self.cur = None
                }
            }
        }
        self.cur.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn monotone_sequences() {
        let ms_2_1 = MonotoneSequences::new(2, 1);
        assert_eq!(ms_2_1.collect::<Vec<_>>(), [[0, 0], [0, 1], [1, 1]]);
        let ms_3_3 = MonotoneSequences::new(3, 3);
        assert_eq!(
            ms_3_3.collect::<Vec<_>>(),
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
                [3, 3, 3]
            ]
        );
    }
}
