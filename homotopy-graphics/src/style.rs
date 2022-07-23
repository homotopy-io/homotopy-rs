use std::{fmt, str::FromStr};

use homotopy_core::Generator;
use serde::{Deserialize, Serialize};

pub trait GeneratorStyle {
    fn label(&self) -> Option<String>;
    fn shape(&self) -> Option<VertexShape>;
    fn color(&self) -> Color;
}

pub trait SignatureStyleData {
    type Style: GeneratorStyle;

    fn generator_style(&self, g: Generator) -> Option<&Self::Style>;

    // It would be nice if the following could be an iterator but the generics get complex fast
    fn generators(&self) -> Vec<Generator>;
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

impl Default for Color {
    fn default() -> Self {
        Self(palette::Srgb::new(0, 0, 0))
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = self.0.into_components();
        write!(f, "#{:02x}{:02x}{:02x}", r, g, b)
    }
}

impl Default for VertexShape {
    fn default() -> Self {
        Self::Circle
    }
}
