use once_cell::unsync::OnceCell;
use serde::{Deserialize, Serialize};

use crate::{Diagram, Rewrite};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Drawable {
    Diagram(Diagram),
    Rewrite(Diagram, Rewrite, Diagram),
}

pub trait Debugger {
    fn debug(&self, drawables: Vec<Drawable>);
}

thread_local! {
    static DEBUGGER: OnceCell<Box<dyn Debugger>> = OnceCell::new();
}

pub fn set_debugger<F>(make_debugger: F)
where
    F: FnOnce() -> Box<dyn Debugger>,
{
    DEBUGGER.with(|debugger| {
        let _res = debugger.set(make_debugger());
    });
}

pub fn debug_diagram(d: Diagram) {
    debug_diagrams(vec![d]);
}

pub fn debug_diagrams(ds: Vec<Diagram>) {
    DEBUGGER.with(|debugger| {
        debugger
            .get()
            .expect("no debugger!")
            .debug(ds.into_iter().map(Drawable::Diagram).collect());
    });
}
