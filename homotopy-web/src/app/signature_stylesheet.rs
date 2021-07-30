use crate::model::proof::{Color, Signature};
use homotopy_core::Generator;

use palette::Lighten;

use std::fmt::Write;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node};

// TODO: Check if there is a performance problem with this. If so, then use the
// stylesheet API to change the colors more granularly.

pub struct SignatureStylesheet {
    signature: Signature,
    element: Element,
    prefix: String,
}

impl SignatureStylesheet {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            signature: Default::default(),
            element: document().create_element("style").unwrap(),
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

        for (generator, info) in self.signature.iter() {
            writeln!(
                style,
                ".{name} {{ fill: {color}; stroke: {color}; }}",
                name = Self::name(&self.prefix, *generator, "surface"),
                color = Color((info.color.into_format().into_linear().lighten(0.1)).into())
            )
            .unwrap();
            writeln!(
                style,
                ".{name} {{ stroke: {color}; }}",
                name = Self::name(&self.prefix, *generator, "wire"),
                color = Color((info.color.into_format().into_linear().lighten(0.05)).into())
            )
            .unwrap();
            writeln!(
                style,
                ".{name} {{ fill: {color}; }}",
                name = Self::name(&self.prefix, *generator, "point"),
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

fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}
