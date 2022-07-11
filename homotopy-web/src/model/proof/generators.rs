use std::{fmt, ops::Deref};

use homotopy_common::hash::FastHashMap;
use homotopy_core::{common::Generator, Diagram};
use palette::Srgb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratorInfo {
    pub generator: Generator,
    pub name: String,
    pub color: Color,
    pub shape: VertexShape,
    pub diagram: Diagram,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub Srgb<u8>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexShape {
    Circle, // circle / sphere
    Square, // square / cube
}

impl Deref for Color {
    type Target = Srgb<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO(thud): This can go soon
impl Default for Color {
    fn default() -> Self {
        Self(Srgb::new(0, 0, 0))
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = self.into_components();
        write!(f, "#{:02x}{:02x}{:02x}", r, g, b)
    }
}

impl Default for VertexShape {
    fn default() -> Self {
        Self::Circle // TODO(thud): have this be decided by the user in settings?
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct GeneratorState(FastHashMap<Generator, GeneratorInfo>);

impl GeneratorState {
    pub fn get(&self, g: Generator) -> Option<&GeneratorInfo> {
        self.0.get(&g)
    }

    pub fn get_mut(&mut self, g: Generator) -> Option<&mut GeneratorInfo> {
        self.0.get_mut(&g)
    }

    pub fn insert(&mut self, g: Generator, i: GeneratorInfo) -> Option<GeneratorInfo> {
        self.0.insert(g, i)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Rename(Generator, String),
    Recolor(Generator, Color),
    Reshape(Generator, VertexShape),
}
