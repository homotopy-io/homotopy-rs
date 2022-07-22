use homotopy_core::Generator;
use homotopy_graphics::style::CssStylesheet;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node};

use crate::{components::document, model::proof::Signature};

// It would be nice if we could remove the `SignatureStylesheet` entirely but we still need to
// maintain a handle to the html node for mounting/unmounting. This also means we can avoid
// regenerating the stylesheet unless the signature has changed [`update`].

// TODO: Check if there is a performance problem with the current approach.. If so, then use the
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
        Signature::css_class(prefix, generator, style)
    }

    fn node(&self) -> Node {
        self.element.clone().dyn_into::<Node>().unwrap()
    }

    pub fn update(&mut self, signature: Signature) {
        if signature != self.signature {
            self.signature = signature;
            self.element
                .set_inner_html(&self.signature.css_stylesheet(&self.prefix));
        }
    }
}
