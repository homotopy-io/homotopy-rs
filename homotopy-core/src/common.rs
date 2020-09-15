use std::fmt;
use serde::{ Serialize, Deserialize, Serializer, Deserializer };

#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Generator {
    pub id: usize,
    pub dimension: usize,
}

impl fmt::Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.id, self.dimension))
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Boundary {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "target")]
    Target,
}

pub type SingularHeight = usize;

pub type RegularHeight = usize;

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

impl Serialize for Height {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        serializer.serialize_u32(self.to_int() as u32)
    }
}

impl<'de> Deserialize<'de> for Height {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        Ok(Height::from_int(u32::deserialize(deserializer)? as usize))
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SliceIndex {
    Boundary(Boundary),
    Interior(Height),
}

impl SliceIndex {
    pub fn to_int(self, size: usize) -> isize {
        match self {
            SliceIndex::Boundary(Boundary::Source) => -1,
            SliceIndex::Boundary(Boundary::Target) => size as isize * 2 + 1,
            SliceIndex::Interior(height) => height.to_int() as isize,
        }
    }

    pub fn from_int(h: isize, size: usize) -> Self {
        if h < 0 {
            Boundary::Source.into()
        } else if h > 2 * size as isize {
            Boundary::Target.into()
        } else {
            Height::from_int(h as usize).into()
        }
    }
}

impl From<Height> for SliceIndex {
    fn from(height: Height) -> Self {
        SliceIndex::Interior(height)
    }
}

impl From<Boundary> for SliceIndex {
    fn from(boundary: Boundary) -> Self {
        SliceIndex::Boundary(boundary)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Forward,
    Backward,
}
