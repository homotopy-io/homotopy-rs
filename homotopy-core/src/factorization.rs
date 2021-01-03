pub struct MonotoneSequences {
    cur: Option<Vec<usize>>,

    // tracks least significant digit in the sequence;
    // once a digit from the right reaches `max` it becomes irrelevant for
    // subsequent sequences
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
                    if seq[self.end] != self.max {
                        seq[self.end] += 1
                    } else {
                        // least significant digit has been maxed
                        self.end -= 1; // 1. make the next digit least significant
                        seq[self.end] += 1; // 2. increment it
                        for i in (self.end + 1)..self.len {
                            // 3. set all values to the right to it
                            seq[i] = seq[self.end]
                        }
                    }
                } else {
                    self.cur = None
                }
            }
        }
        self.cur.clone()
    }
}

mod test {
    use super::*;

    #[test]
    fn monotone_sequences() {
        let ms = MonotoneSequences::new(2, 1);
        assert_eq!(ms.collect::<Vec<_>>(), [[0, 0], [0, 1], [1, 1]])
    }
}
