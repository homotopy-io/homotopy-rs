use std::iter::FusedIterator;

/// An iterator that returns either no items, one item, or the items of another iterator.
#[derive(Clone)]
pub enum ZeroOneMany<T: Iterator> {
    Empty,
    One(T::Item),
    Many(T),
}

impl<T: Iterator> Default for ZeroOneMany<T> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<T: Iterator> Iterator for ZeroOneMany<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Self::Many(iter) = self {
            iter.next()
        } else {
            match std::mem::take(self) {
                Self::Empty => None,
                Self::One(x) => Some(x),
                Self::Many(_) => unreachable!(),
            }
        }
    }
}

impl<T: FusedIterator> FusedIterator for ZeroOneMany<T> {}
