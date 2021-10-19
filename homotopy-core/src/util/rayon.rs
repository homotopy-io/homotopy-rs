use std::cell::Cell;

#[cfg(not(feature = "rayon"))]
pub use itertools::Itertools;
#[cfg(feature = "rayon")]
pub use rayon::join;
#[cfg(feature = "rayon")]
pub use rayon::prelude::*;
#[cfg(not(feature = "rayon"))]
pub use stub::*;

#[cfg(not(feature = "rayon"))]
mod stub {
    // single-threaded rayon stub
    pub fn join<A, B, RA, RB>(f: A, g: B) -> (RA, RB)
    where
        A: FnOnce() -> RA,
        B: FnOnce() -> RB,
    {
        (f(), g())
    }

    pub trait SequentialIterator<'a, I: Iterator> {
        fn par_iter(&'a self) -> I;
    }

    impl<'a, T> SequentialIterator<'a, std::slice::Iter<'a, T>> for Vec<T> {
        fn par_iter(&'a self) -> std::slice::Iter<'a, T> {
            self.iter()
        }
    }

    impl<'a, T> SequentialIterator<'a, std::slice::Iter<'a, T>> for [T] {
        fn par_iter(&'a self) -> std::slice::Iter<'a, T> {
            self.iter()
        }
    }

    pub trait IntoSequentialIterator<I: Iterator> {
        fn into_par_iter(self) -> I;
    }

    impl<T: IntoIterator> IntoSequentialIterator<T::IntoIter> for T {
        fn into_par_iter(self) -> T::IntoIter {
            self.into_iter()
        }
    }
}

thread_local! {
    pub static MAIN_THREAD: Cell<bool> = Cell::new(false);
}

#[allow(unused_macros)]
macro_rules! background_only {
    ($body:expr) => {
        #[cfg(feature = "rayon")]
        MAIN_THREAD.with(|main| {
            if !main.get() {
                $body
            }
        });
        #[cfg(not(feature = "rayon"))]
        $body
    };
}

#[allow(unused_imports)]
pub(crate) use background_only;
