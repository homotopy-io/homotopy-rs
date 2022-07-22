use std::fmt::Write;

use homotopy_core::Generator;

pub trait GeneratorStyle {
    type Color: RenderableColor;

    fn label(&self) -> Option<String>;
    fn shape(&self) -> Option<VertexShape>;
    fn color_point(&self) -> Self::Color;
    fn color_wire(&self) -> Self::Color;
    fn color_surface(&self) -> Self::Color;
}

pub trait SignatureStyleData {
    type Style: GeneratorStyle;

    fn generator_style(&self, g: Generator) -> Option<&Self::Style>;

    // It would be nice if the following could be an iterator but the generics get complex fast
    fn generators(&self) -> Vec<Generator>;
}

pub enum VertexShape {
    Circle,
    Square,
}

impl Default for VertexShape {
    fn default() -> Self {
        Self::Circle
    }
}

pub trait RenderableColor {
    fn css(&self) -> String;
}

pub trait CssStylesheet {
    fn css_stylesheet(&self, prefix: &str) -> String;
    fn css_class(prefix: &str, generator: Generator, suffix: &str) -> String;
}

impl<StyleData: SignatureStyleData> CssStylesheet for StyleData {
    fn css_stylesheet(&self, prefix: &str) -> String {
        let mut stylesheet = String::new();

        for generator in self.generators() {
            let style = self.generator_style(generator).unwrap();

            writeln!(
                stylesheet,
                ".{name} {{ fill: {color}; stroke: {color}; }}",
                name = Self::css_class(prefix, generator, "surface"),
                color = style.color_surface().css()
            )
            .unwrap();
            writeln!(
                stylesheet,
                ".{name} {{ stroke: {color}; }}",
                name = Self::css_class(prefix, generator, "wire"),
                color = style.color_wire().css()
            )
            .unwrap();
            writeln!(
                stylesheet,
                ".{name} {{ fill: {color}; }}",
                name = Self::css_class(prefix, generator, "point"),
                color = style.color_point().css()
            )
            .unwrap();
        }

        stylesheet
    }

    fn css_class(prefix: &str, generator: Generator, suffix: &str) -> String {
        format!(
            "{}__{}-{}--{}",
            prefix, generator.id, generator.dimension, suffix
        )
    }
}
