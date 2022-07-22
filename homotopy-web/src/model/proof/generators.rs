use std::{fmt, ops::Deref};

use homotopy_core::{common::Generator, Diagram};
use homotopy_graphics::{
    style,
    style::{GeneratorStyle, RenderableColor, SignatureStyleData},
};
use palette::{Lighten, Srgb};
use serde::{Deserialize, Serialize};

use super::signature::Signature;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratorInfo {
    pub generator: Generator,
    pub name: String,
    pub framed: bool,
    pub invertible: bool,
    pub color: Color,
    pub shape: VertexShape,
    pub diagram: Diagram,
}

impl SignatureStyleData for Signature {
    type Style = GeneratorInfo;

    fn generators(&self) -> Vec<Generator> {
        self.iter().map(|info| info.generator).collect()
    }

    fn generator_style(&self, g: Generator) -> Option<&Self::Style> {
        self.generator_info(g)
    }
}

impl GeneratorStyle for GeneratorInfo {
    type Color = Color;

    fn label(&self) -> Option<String> {
        // TODO(thud): Decide whether to show a label
        // Some(self.name.clone())
        None
    }

    fn shape(&self) -> Option<style::VertexShape> {
        Some(self.shape.clone().into())
    }

    fn color_point(&self) -> Self::Color {
        self.color.clone()
    }

    fn color_wire(&self) -> Self::Color {
        Color((self.color.into_linear().lighten(0.05)).into())
    }

    fn color_surface(&self) -> Self::Color {
        Color((self.color.into_linear().lighten(0.1)).into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub Srgb<u8>);

impl RenderableColor for Color {
    fn css(&self) -> String {
        format!("#{:X}", self.0)
    }

    fn tikz(&self) -> String {
        format!(
            "{{RGB}}{{{r}, {g}, {b}}}",
            r = self.red,
            g = self.green,
            b = self.blue
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexShape {
    Circle, // circle / sphere
    Square, // square / cube
}

impl From<VertexShape> for style::VertexShape {
    fn from(shape: VertexShape) -> Self {
        use VertexShape::{Circle, Square};
        match shape {
            Circle => Self::Circle,
            Square => Self::Square,
        }
    }
}

impl Deref for Color {
    type Target = Srgb<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
        Self::Circle // TODO(thud): have this be decided by the user in settings UI
    }
}
