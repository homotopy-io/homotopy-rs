use std::cell::RefCell;

use homotopy_common::hash::FastHashMap;
use thiserror::Error;

use crate::{
    common::Mode,
    diagram::RewritingError,
    rewrite::{CompositionError, ConeInternal},
    Cospan, Diagram, DiagramN, Direction, Height, Rewrite, Rewrite0, RewriteN,
};

thread_local! {
    static DIAGRAM_CACHE: RefCell<FastHashMap<DiagramN, Vec<MalformedDiagram>>> = RefCell::new(FastHashMap::default());
    static REWRITE_CACHE: RefCell<FastHashMap<RewriteN, Vec<MalformedRewrite>>> = RefCell::new(FastHashMap::default());
}

impl Diagram {
    pub fn check(&self, mode: Mode) -> Result<(), Vec<MalformedDiagram>> {
        let result = self.check_worker(mode);

        // Clear cache
        DIAGRAM_CACHE.with(|cache| cache.borrow_mut().clear());
        REWRITE_CACHE.with(|cache| cache.borrow_mut().clear());

        result
    }

    fn check_worker(&self, mode: Mode) -> Result<(), Vec<MalformedDiagram>> {
        match self {
            Self::Diagram0(_) => Ok(()),
            Self::DiagramN(d) => d.check_worker(mode),
        }
    }
}

impl DiagramN {
    pub fn check(&self, mode: Mode) -> Result<(), Vec<MalformedDiagram>> {
        let result = self.check_worker(mode);

        // Clear cache
        DIAGRAM_CACHE.with(|cache| cache.borrow_mut().clear());
        REWRITE_CACHE.with(|cache| cache.borrow_mut().clear());

        result
    }

    fn check_worker(&self, mode: Mode) -> Result<(), Vec<MalformedDiagram>> {
        let cached = DIAGRAM_CACHE.with(|cache| cache.borrow().get(self).cloned());
        if let Some(errors) = cached {
            return if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            };
        }

        let mut slice = self.source();
        let mut errors: Vec<MalformedDiagram> = Vec::new();

        // Check that the source slice is well-formed.
        if mode == Mode::Deep {
            if let Err(e) = slice.check_worker(mode) {
                errors.push(MalformedDiagram::Slice(Height::Regular(0), e));
            }
        }

        for (i, cospan) in self.cospans().iter().enumerate() {
            // Check that the forward rewrite is well-formed.
            if mode == Mode::Deep {
                if let Err(e) = cospan.forward.check_worker(mode) {
                    errors.push(MalformedDiagram::Rewrite(i, Direction::Forward, e));
                }
            }

            // Check that the forward rewrite is compatible with the regular slice.
            match slice.rewrite_forward(&cospan.forward) {
                Ok(d) => slice = d,
                Err(re) => {
                    errors.push(MalformedDiagram::Incompatible(i, Direction::Forward, re));
                    break;
                }
            }

            // Check that the singular slice is well-formed.
            if mode == Mode::Deep {
                if let Err(e) = slice.check_worker(mode) {
                    errors.push(MalformedDiagram::Slice(Height::Singular(i), e));
                }
            }

            // Check that the backward rewrite is well-formed.
            if mode == Mode::Deep {
                if let Err(e) = cospan.backward.check_worker(mode) {
                    errors.push(MalformedDiagram::Rewrite(i, Direction::Backward, e));
                }
            }

            // Check that the backward rewrite is compatible with the singular slice.
            match slice.rewrite_backward(&cospan.backward) {
                Ok(d) => slice = d,
                Err(re) => {
                    errors.push(MalformedDiagram::Incompatible(i, Direction::Backward, re));
                    break;
                }
            }

            // Check that the regular slice is well-formed.
            if mode == Mode::Deep {
                if let Err(e) = slice.check_worker(mode) {
                    errors.push(MalformedDiagram::Slice(Height::Regular(i + 1), e));
                }
            }
        }

        DIAGRAM_CACHE.with(|cache| cache.borrow_mut().insert(self.clone(), errors.clone()));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Rewrite {
    pub fn check(&self, mode: Mode) -> Result<(), Vec<MalformedRewrite>> {
        let result = self.check_worker(mode);

        // Clear cache
        DIAGRAM_CACHE.with(|cache| cache.borrow_mut().clear());
        REWRITE_CACHE.with(|cache| cache.borrow_mut().clear());

        result
    }

    fn check_worker(&self, mode: Mode) -> Result<(), Vec<MalformedRewrite>> {
        match self {
            Self::Rewrite0(_) => Ok(()),
            Self::RewriteN(r) => r.check_worker(mode),
        }
    }
}

impl RewriteN {
    pub fn check(&self, mode: Mode) -> Result<(), Vec<MalformedRewrite>> {
        let result = self.check_worker(mode);

        // Clear cache
        DIAGRAM_CACHE.with(|cache| cache.borrow_mut().clear());
        REWRITE_CACHE.with(|cache| cache.borrow_mut().clear());

        result
    }

    fn check_worker(&self, mode: Mode) -> Result<(), Vec<MalformedRewrite>> {
        let cached = REWRITE_CACHE.with(|cache| cache.borrow().get(self).cloned());
        if let Some(errors) = cached {
            return if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            };
        }

        let mut errors: Vec<MalformedRewrite> = Vec::new();
        for (i, cone) in self.cones().iter().enumerate() {
            if mode == Mode::Deep {
                // Check that the regular slices are well-formed.
                for (j, slice) in cone.regular_slices().iter().enumerate() {
                    if let Err(e) = slice.check_worker(mode) {
                        errors.push(MalformedRewrite::RegularSlice(i, j, e));
                    }
                }

                // Check that the singular slices are well-formed.
                for (j, slice) in cone.singular_slices().iter().enumerate() {
                    if let Err(e) = slice.check_worker(mode) {
                        errors.push(MalformedRewrite::SingularSlice(i, j, e));
                    }
                }
            }

            // Check commutativity conditions.
            match cone.internal.get() {
                ConeInternal::Cone0 {
                    target,
                    regular_slice,
                } => {
                    if !regular_slice.agrees_with(&target.forward) {
                        errors.push(MalformedRewrite::NotCommutativeUnit(i));
                    }

                    if !regular_slice.agrees_with(&target.backward) {
                        errors.push(MalformedRewrite::NotCommutativeUnit(i));
                    }
                }
                ConeInternal::ConeN {
                    source,
                    target,
                    regular_slices,
                    singular_slices,
                } => {
                    match source
                        .first()
                        .unwrap()
                        .forward
                        .compose(&singular_slices.first().unwrap())
                    {
                        Ok(f) if f.agrees_with(&target.forward) => { /* no error */ }
                        Ok(_) => errors.push(MalformedRewrite::NotCommutative(i, 0)),
                        Err(ce) => errors.push(ce.into()),
                    };

                    for (j, regular_slice) in regular_slices.iter().enumerate() {
                        match source[j].backward.compose(&singular_slices[j]) {
                            Ok(f) if f.agrees_with(&regular_slice) => { /* no error */ }
                            Ok(_) => errors.push(MalformedRewrite::NotCommutative(i, j + 1)),
                            Err(ce) => errors.push(ce.into()),
                        }

                        match source[j + 1].forward.compose(&singular_slices[j + 1]) {
                            Ok(f) if f.agrees_with(&regular_slice) => { /* no error */ }
                            Ok(_) => errors.push(MalformedRewrite::NotCommutative(i, j + 1)),
                            Err(ce) => errors.push(ce.into()),
                        }
                    }

                    match source
                        .last()
                        .unwrap()
                        .backward
                        .compose(&singular_slices.last().unwrap())
                    {
                        Ok(f) if f.agrees_with(&target.backward) => { /* no error */ }
                        Ok(_) => errors.push(MalformedRewrite::NotCommutative(i, cone.len())),
                        Err(ce) => errors.push(ce.into()),
                    };
                }
            }
        }

        REWRITE_CACHE.with(|cache| cache.borrow_mut().insert(self.clone(), errors.clone()));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Clone, Debug, Error)]
pub enum MalformedDiagram {
    #[error("slice {0:?} is malformed: {1:?}")]
    Slice(Height, Vec<MalformedDiagram>),

    #[error("rewrite {0} in direction {1:?} is malformed: {2:?}")]
    Rewrite(usize, Direction, Vec<MalformedRewrite>),

    #[error("rewrite {0} in direction {1:?} is incompatible with its source: {2:?}")]
    Incompatible(usize, Direction, RewritingError),
}

#[derive(Clone, Debug, Error)]
pub enum MalformedRewrite {
    #[error(transparent)]
    Composition(#[from] CompositionError),

    #[error("regular slice {1} of cone {0} is malformed: {2:?}")]
    RegularSlice(usize, usize, Vec<MalformedRewrite>),

    #[error("singular slice {1} of cone {0} is malformed: {2:?}")]
    SingularSlice(usize, usize, Vec<MalformedRewrite>),

    #[error("unit cone {0} fails to be commutative.")]
    NotCommutativeUnit(usize),

    #[error("cone {0} fails to be commutative at index {1}.")]
    NotCommutative(usize, usize),
}

impl Rewrite {
    /// Checks if `self` agrees with `other` ignoring labels.
    fn agrees_with(&self, other: &Self) -> bool {
        use Rewrite::{Rewrite0, RewriteN};
        match (self, other) {
            (Rewrite0(f), Rewrite0(g)) => f.agrees_with(g),
            (RewriteN(f), RewriteN(g)) => f.agrees_with(g),
            _ => false,
        }
    }
}

impl Rewrite0 {
    fn agrees_with(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some((f_s, f_t, _, _)), Some((g_s, g_t, _, _))) => *f_s == *g_s && *f_t == *g_t,
            _ => false,
        }
    }
}

impl RewriteN {
    fn agrees_with(&self, other: &Self) -> bool {
        if self.cones().len() != other.cones().len() {
            return false;
        }
        std::iter::zip(self.cones(), other.cones()).all(|(f_cone, g_cone)| {
            f_cone.index == g_cone.index
                && f_cone.len() == g_cone.len()
                && std::iter::zip(f_cone.source(), g_cone.source())
                    .all(|(f_cs, g_cs)| f_cs.agrees_with(g_cs))
                && f_cone.target().agrees_with(g_cone.target())
                && std::iter::zip(f_cone.regular_slices(), g_cone.regular_slices())
                    .all(|(f_slice, g_slice)| f_slice.agrees_with(g_slice))
                && std::iter::zip(f_cone.singular_slices(), g_cone.singular_slices())
                    .all(|(f_slice, g_slice)| f_slice.agrees_with(g_slice))
        })
    }
}

impl Cospan {
    fn agrees_with(&self, other: &Self) -> bool {
        self.forward.agrees_with(&other.forward) && self.backward.agrees_with(&other.backward)
    }
}
