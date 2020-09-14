#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash)]
pub struct Generator {
    pub id: usize,
    pub dimension: usize,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash)]
pub enum Boundary {
    Source,
    Target,
}

type SingularHeight = usize;

type RegularHeight = usize;

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash)]
pub enum Height {
    Singular(SingularHeight),
    Regular(RegularHeight),
}

impl Height {
    pub fn to_int(self) -> usize {
        match self {
            Height::Regular(i) => i * 2,
            Height::Singular(i) => i * 2 + 1,
        }
    }

    pub fn from_int(h: usize) -> Height {
        if h % 2 == 0 {
            Height::Regular(h / 2)
        } else {
            Height::Singular((h - 1) / 2)
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash)]
pub enum SliceIndex {
    Boundary(Boundary),
    Interior(Height),
}
