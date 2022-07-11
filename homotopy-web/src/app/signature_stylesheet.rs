use std::fmt::Write;

use homotopy_core::Generator;
use palette::Lighten;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node};

use crate::{
    components::document,
    model::proof::{generators::Color, Signature},
};

// TODO: Check if there is a performance problem with this. If so, then use the
// stylesheet API to change the colors more granularly.

pub struct SignatureStylesheet {
    signature: Signature,
    element: Element,
    prefix: String,
}

impl SignatureStylesheet {
    pub fn new(prefix: impl Into<String>) -> Self {
        let element = document().create_element("style").unwrap();
        element.set_id("signature__stylesheet");
        Self {
            signature: Default::default(),
            element,
            prefix: prefix.into(),
        }
    }

    pub fn mount(&self) {
        document()
            .head()
            .unwrap()
            .append_child(&self.node())
            .unwrap();
    }

    pub fn unmount(&self) {
        document()
            .head()
            .unwrap()
            .remove_child(&self.node())
            .unwrap();
    }

    pub fn name(prefix: &str, generator: Generator, style: &str) -> String {
        format!(
            "{}__{}-{}--{}",
            prefix, generator.id, generator.dimension, style
        )
    }

    fn node(&self) -> Node {
        self.element.clone().dyn_into::<Node>().unwrap()
    }

    fn style(&self) -> String {
        let mut style = String::new();

        for info in self.signature.iter() {
            writeln!(
                style,
                ".{name} {{ fill: {color}; stroke: {color}; }}",
                name = Self::name(&self.prefix, info.generator, "surface"),
                color = Color((info.color.into_linear().lighten(0.1)).into())
            )
            .unwrap();
            writeln!(
                style,
                ".{name} {{ stroke: {color}; }}",
                name = Self::name(&self.prefix, info.generator, "wire"),
                color = Color((info.color.into_linear().lighten(0.05)).into())
            )
            .unwrap();
            writeln!(
                style,
                ".{name} {{ fill: {color}; }}",
                name = Self::name(&self.prefix, info.generator, "point"),
                color = info.color
            )
            .unwrap();
        }

        style
    }

    pub fn update(&mut self, signature: Signature) {
        if signature != self.signature {
            self.signature = signature;
            self.element.set_inner_html(&self.style());
        }
    }
}
