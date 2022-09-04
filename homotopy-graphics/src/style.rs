use std::{fmt, str::FromStr};

use homotopy_core::{Generator, Orientation};
use palette::{convert::FromColor, Hsl, Srgb};
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
pub struct Color(pub(crate) Srgb<u8>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexShape {
    Circle, // circle / sphere
    Square, // square / cube
}

impl Color {
    const MIN_LIGHTNESS_WRAP: f32 = 0.25;
    const MAX_LIGHTNESS_WRAP: f32 = 0.90;

    pub fn hex(&self) -> String {
        let (r, g, b) = self.clone().into_components::<u8>();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    // Used for UI to make sure we always maintain sufficient contrast for legibility.
    pub fn is_light(&self) -> bool {
        palette::RelativeContrast::get_contrast_ratio(
            palette::Srgb::new(1., 1., 1.),
            self.0.into_format::<f32>(),
        ) < 1.5
    }

    // Get color in lightened form. We wrap colors if needed so that they are sufficiently
    // distinguishable for end users.
    //
    // Colors are distinguished as pairs (C, R) where:
    //      C = max(0, D - N - K)
    //      R = -1 | 0 | 1          // orientation of generator
    //      D = [usize]             // diagram dimension
    //      N = [usize]             // generator dimension
    //      K = 0 | 1 | 2           // point | wire | surface (in visible diagram)
    //
    // See: https://github.com/homotopy-io/homotopy-rs/issues/550
    //
    // We treat three kinds of color differently (depending on initial lightness):
    //      lightness < MIN_LIGHTNESS_WRAP
    //          => lighten colours (zero then inverse)
    //      lightness > MAX_LIGHTNESS_WRAP
    //          => lighten colours (zero then inverse) wrapping such that min lightness is never
    //             below MIN_LIGHTNESS_WRAP
    //      otherwise
    //          => lighten colours (zero then inverse) wrapping before reaching MAX_LIGHTNESS_WRAP
    //             and respecting MIN_LIGHTNESS_WRAP
    //
    // The constants used are fairly arbitrary and subject to tweaking.
    #[inline]
    #[must_use]
    pub fn lighten(&self, c: usize, orientation: Orientation) -> Self {
        let mut hsl: Hsl = FromColor::from_color(self.0.into_format::<f32>());
        let (min_lightness, max_lightness) = if hsl.lightness < Self::MIN_LIGHTNESS_WRAP {
            (0., 1.)
        } else if hsl.lightness > Self::MAX_LIGHTNESS_WRAP {
            (Self::MIN_LIGHTNESS_WRAP, 1.)
        } else {
            (Self::MIN_LIGHTNESS_WRAP, Self::MAX_LIGHTNESS_WRAP)
        };
        let r = match orientation {
            Orientation::Positive => 1,
            Orientation::Zero => 0,
            Orientation::Negative => -1,
        };
        let offset = 3 * (1 - r) + (c as isize);
        let o = 0.08 * offset as f32;
        hsl.lightness = (hsl.lightness + o - min_lightness - 0.01)
            % (max_lightness - min_lightness)
            + min_lightness
            + 0.01;
        let srgb: Srgb<f32> = FromColor::from_color(hsl);
        Self(srgb.into_format())
    }

    pub fn into_components<T>(self) -> (T, T, T)
    where
        T: palette::stimulus::FromStimulus<u8>,
    {
        self.0.into_format().into_components()
    }

    pub fn into_linear_f32_components(self) -> (f32, f32, f32) {
        self.0.into_format::<f32>().into_linear().into_components()
    }
}

// Convert from hex string (#rrggbb) to `Color`
impl FromStr for Color {
    type Err = palette::rgb::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        palette::Srgb::<u8>::from_str(s)
            .map(palette::Srgb::into_format)
            .map(Self)
    }
}

// Reverse of `FromStr` (#rrggbb)
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hex())
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

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum GeneratorRepresentation {
    Point = 0,
    Wire = 1,
    Surface = 2,
}

impl std::fmt::Display for GeneratorRepresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Point => "point",
                Self::Wire => "wire",
                Self::Surface => "surface",
            }
        )
    }
}
