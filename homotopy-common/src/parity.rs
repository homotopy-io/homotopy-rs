use std::cmp::Ordering;

macro_rules! inline_bubble  {
    ($f:expr, $t:expr, $($i:literal),*$(,)*) => {{
        let f = ($f);
        let t = ($t);

        let mut parity = true;

        $(if f(t[$i + 1], t[$i]) == Ordering::Less {
            t.swap($i, $i + 1);
            parity = !parity;
        })*

        parity
    }};
}

#[inline]
pub fn sort_2<F, T>(t: &mut [T; 2], f: F) -> bool
where
    F: Fn(T, T) -> Ordering,
    T: Copy,
{
    inline_bubble!(f, t, 0)
}

#[inline]
pub fn sort_3<F, T>(t: &mut [T; 3], f: F) -> bool
where
    F: Fn(T, T) -> Ordering,
    T: Copy,
{
    inline_bubble!(f, t, 0, 1, 0)
}

#[inline]
pub fn sort_4<F, T>(t: &mut [T; 4], f: F) -> bool
where
    F: Fn(T, T) -> Ordering,
    T: Copy,
{
    inline_bubble!(f, t, 0, 1, 2, 0, 1, 0)
}
