use std::{fmt, str::FromStr};

use homotopy_core::Generator;
use palette::{convert::FromColor, Hsl, Lighten, Srgb};
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
    pub fn hex(&self) -> String {
        let (r, g, b) = self.clone().into_components::<u8>();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    #[must_use]
    pub fn lighten(&self, amount: f32) -> Self {
        Self(self.0.into_linear().lighten(amount).into())
    }

    #[must_use]
    pub fn with_lightness(&self, lightness: f32) -> Self {
        let mut hsl: Hsl = FromColor::from_color(self.0.into_format::<f32>());
        hsl.lightness = lightness;
        let srgb: Srgb<f32> = FromColor::from_color(hsl);
        Self(srgb.into_format())
    }

    // we combine saturate and lighten into a single function since we need to convert SRGB into
    // HSL or HSV color space anyway and we save some computation by lightening this directly
    // instead of the resulting SRGB.
    // #[must_use]
    // pub fn desaturate_and_lighten(&self, desaturate: f32, lighten: f32) -> Self {
    //     let hsl: Hsl = FromColor::from_color(self.0.into_format::<f32>());
    //     let desaturated = Saturate::saturate(hsl, -desaturate);
    //     let lightened = lighten(desaturated, lighten);
    //     let srgb: palette::Srgb<f32> = palette::convert::FromColor::from_color(lightened);
    //     Self(srgb.into_format())
    // }

    pub fn is_light(&self) -> bool {
        palette::RelativeContrast::get_contrast_ratio(
            palette::Srgb::new(1., 1., 1.),
            self.0.into_format::<f32>(),
        ) < 1.5
    }

    pub fn is_dark(&self) -> bool {
        palette::RelativeContrast::get_contrast_ratio(
            palette::Srgb::new(0., 0., 0.),
            self.0.into_format::<f32>(),
        ) < 1.5
    }

    // Get color in lightened form (based on offset [0, 8]). We wrap colors if needed so that they
    // are sufficiently different and clearly distinguishable for end users.
    //
    // Colours are calculated from C and R where:
    //      C = max(0, D - N - K)
    //      R = -1 | 0 | 1          // orientation of generator
    //      D = 0...                // diagram dimension
    //      N = 0...                // generator dimension
    //      K = 0 | 1 | 2           // point | wire | surface (in visible diagram)
    //
    // See: https://github.com/homotopy-io/homotopy-rs/issues/550
    //
    // We treat three different kinds of color differently:
    //      self.is_light() == true     => darken colours (zero then inverse)
    //      self.is_dark() == true      => lighten colours (zero then inverse)
    //      otherwise                   => lighten zero, darken inverse
    // This avoids unnecessary wrapping of colours as it can be jarring or unclear. The constants
    // used are fairly arbitrary and subject to tweaking to get diagrams looking good.
    #[inline]
    #[must_use]
    pub fn lighten_from_c_r(&self, c: isize, r: isize) -> Self {
        let mut hsl: Hsl = FromColor::from_color(self.0.into_format::<f32>());
        let raw_offset = 3 * (1 - r) + c;
        let offset = if !self.is_light() && !self.is_dark() && raw_offset > 5 {
            raw_offset - 9
        } else {
            raw_offset
        };
        let dir = if self.is_light() { -1. } else { 1. };
        let o = dir * 0.08 * offset as f32;
        hsl.lightness = (hsl.lightness + o - 0.01) % 1. + 0.01;
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
