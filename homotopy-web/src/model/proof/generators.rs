use std::{fmt, ops::Deref};

use euclid::default::Point2D;
use homotopy_core::{common::Generator, Diagram};
use homotopy_graphics::tikz::{TikzGeneratorStyle, TikzGeneratorStyleAvailable};
use palette::Srgb;
use serde::{Deserialize, Serialize};

use super::signature::Signature;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratorInfo {
    pub generator: Generator,
    pub name: String,
    pub color: Color,
    pub shape: VertexShape,
    pub diagram: Diagram,
}

impl TikzGeneratorStyleAvailable<GeneratorInfo> for Signature {
    fn generator_style(&self, g: Generator) -> Option<&GeneratorInfo> {
        self.generator_info(g)
    }
}

impl TikzGeneratorStyle for GeneratorInfo {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn color(&self) -> String {
        format!(
            "{{RGB}}{{{r}, {g}, {b}}}",
            r = self.color.red,
            g = self.color.green,
            b = self.color.blue,
        )
    }

    fn shape(&self) -> &'static str {
        use VertexShape::{Circle, Square};
        match self.shape {
            Circle => "circle",
            Square => "rectangle",
        }
    }

    fn render(&self, point: Point2D<f32>) -> String {
        use VertexShape::{Circle, Square};
        let (xo, yo) = match self.shape {
            Circle => (0.0, 0.0),
            Square => (-14.0, -14.0),
        };
        let x1 = (point.x * 100.0 + xo).round() / 100.0;
        let y1 = (point.y * 100.0 + yo).round() / 100.0;
        // TODO(thud): the below should not need allocate Vecs [though export speed is unlikely to
        // be important to the user]
        let sz = match self.shape {
            Circle => vec![0.14],                 // r = 4pt
            Square => vec![0.28 + x1, 0.28 + y1], // 8pt x 8pt
        }
        .iter()
        .map(|&s| s.to_string())
        .collect::<Vec<String>>()
        .join(", ");
        format!("({},{}) {} ({});", x1, y1, self.shape(), sz)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub Srgb<u8>);

// We derive repr(u8) here to allow the passing of vertex shapes into homotopy-graphics (as u8)
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexShape {
    Circle = 0, // circle / sphere
    Square = 1, // square / cube
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Rename(Generator, String),
    Recolor(Generator, Color),
    Reshape(Generator, VertexShape),
}
