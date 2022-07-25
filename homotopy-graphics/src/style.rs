use std::{fmt, str::FromStr};

use homotopy_core::Generator;
use serde::{Deserialize, Serialize};

pub trait GeneratorStyle {
    fn color(&self) -> Color;
    fn label(&self) -> Option<String>;
    fn shape(&self) -> Option<VertexShape>;
}

pub trait SignatureStyleData {
    type Style: GeneratorStyle;

    fn as_pairs(&self) -> Vec<(Generator, &Self::Style)>;
    fn generator_style(&self, g: Generator) -> Option<&Self::Style>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub(crate) palette::Srgb<u8>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexShape {
    Circle, // circle / sphere
    Square, // square / cube
}

impl Color {
    #[must_use]
    pub fn lighten(&self, amount: f32) -> Self {
        Self(palette::Lighten::lighten(self.0.into_linear(), amount).into())
    }

    pub fn into_components<T>(self) -> (T, T, T)
    where
        T: palette::stimulus::FromStimulus<u8>,
    {
        self.0.into_format().into_components()
    }
}

// Convert from hex string (#RRGGBB) to `Color`
impl FromStr for Color {
    type Err = palette::rgb::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        palette::Srgb::<u8>::from_str(s).map(Self)
    }
}

// Reverse of `FromStr`
// NOTE: If we decide to change how we `fmt::Display`, it could break some styles as it is used
// functionally by the css and manim renderers.
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = self.clone().into_components::<u8>();
        write!(f, "#{:02x}{:02x}{:02x}", r, g, b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self(palette::Srgb::new(0, 0, 0))
    }
}

impl Default for VertexShape {
    fn default() -> Self {
        Self::Circle
    }
}
