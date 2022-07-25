use homotopy_graphics::svg;
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
}

impl SignatureStylesheet {
    pub fn new() -> Self {
        let element = document().create_element("style").unwrap();
        element.set_id("signature__stylesheet");
        Self {
            signature: Default::default(),
            element,
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

    fn node(&self) -> Node {
        self.element.clone().dyn_into::<Node>().unwrap()
    }

    pub fn update(&mut self, signature: Signature) {
        if signature != self.signature {
            self.signature = signature;
            self.element
                .set_inner_html(&svg::stylesheet(&self.signature));
        }
    }
}
