use std::{fmt, ops::Deref};

use homotopy_core::{common::Generator, Diagram};
use palette::Srgb;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratorInfo {
    pub generator: Generator,
    pub name: String,
    pub color: Color,
    pub shape: VertexShape,
    pub diagram: Diagram,
}
