use std::cell::RefCell;

use homotopy_common::hash::FastHashMap;
use thiserror::Error;

use crate::{
    diagram::RewritingError,
    rewrite::{CompositionError, Cone},
    Diagram, DiagramN, Direction, Height, Rewrite, Rewrite0, RewriteN,
};

thread_local! {
    static DIAGRAM_CACHE: RefCell<FastHashMap<DiagramN, Vec<MalformedDiagram>>> = RefCell::new(FastHashMap::default());
    static REWRITE_CACHE: RefCell<FastHashMap<RewriteN, Vec<MalformedRewrite>>> = RefCell::new(FastHashMap::default());
}

impl Diagram {
    pub fn check(&self, recursive: bool) -> Result<(), Vec<MalformedDiagram>> {
        let result = self.check_worker(recursive);

        // Clear cache
        DIAGRAM_CACHE.with_borrow_mut(FastHashMap::clear);
        REWRITE_CACHE.with_borrow_mut(FastHashMap::clear);

        result
    }

    fn check_worker(&self, recursive: bool) -> Result<(), Vec<MalformedDiagram>> {
        match self {
            Self::Diagram0(_) => Ok(()),
            Self::DiagramN(d) => d.check_worker(recursive),
        }
    }
}

impl DiagramN {
    pub fn check(&self, recursive: bool) -> Result<(), Vec<MalformedDiagram>> {
        let result = self.check_worker(recursive);

        // Clear cache
        DIAGRAM_CACHE.with_borrow_mut(FastHashMap::clear);
        REWRITE_CACHE.with_borrow_mut(FastHashMap::clear);

        result
    }

    fn check_worker(&self, recursive: bool) -> Result<(), Vec<MalformedDiagram>> {
        if let Some(errors) = DIAGRAM_CACHE.with_borrow(|cache| cache.get(self).cloned()) {
            return if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            };
        }

        let mut slice = self.source();
        let mut errors: Vec<MalformedDiagram> = Vec::new();

        // Check that the source slice is well-formed.
        if recursive {
            if let Err(e) = slice.check_worker(recursive) {
                errors.push(MalformedDiagram::Slice(Height::Regular(0), e));
            }
        }

        for (i, cospan) in self.cospans().iter().enumerate() {
            // Check that the forward rewrite is well-formed.
            if recursive {
                if let Err(e) = cospan.forward.check_worker(recursive) {
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
            if recursive {
                if let Err(e) = slice.check_worker(recursive) {
                    errors.push(MalformedDiagram::Slice(Height::Singular(i), e));
                }
            }

            // Check that the backward rewrite is well-formed.
            if recursive {
                if let Err(e) = cospan.backward.check_worker(recursive) {
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
            if recursive {
                if let Err(e) = slice.check_worker(recursive) {
                    errors.push(MalformedDiagram::Slice(Height::Regular(i + 1), e));
                }
            }
        }

        DIAGRAM_CACHE.with_borrow_mut(|cache| cache.insert(self.clone(), errors.clone()));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Rewrite {
    pub fn check(&self, recursive: bool) -> Result<(), Vec<MalformedRewrite>> {
        let result = self.check_worker(recursive);

        // Clear cache
        DIAGRAM_CACHE.with_borrow_mut(FastHashMap::clear);
        REWRITE_CACHE.with_borrow_mut(FastHashMap::clear);

        result
    }

    fn check_worker(&self, recursive: bool) -> Result<(), Vec<MalformedRewrite>> {
        match self {
            Self::Rewrite0(_) => Ok(()),
            Self::RewriteN(r) => r.check_worker(recursive),
        }
    }
}

impl RewriteN {
    pub fn check(&self, recursive: bool) -> Result<(), Vec<MalformedRewrite>> {
        let result = self.check_worker(recursive);

        // Clear cache
        DIAGRAM_CACHE.with_borrow_mut(FastHashMap::clear);
        REWRITE_CACHE.with_borrow_mut(FastHashMap::clear);

        result
    }

    fn check_worker(&self, recursive: bool) -> Result<(), Vec<MalformedRewrite>> {
        if let Some(errors) = REWRITE_CACHE.with_borrow(|cache| cache.get(self).cloned()) {
            return if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            };
        }

        let mut errors: Vec<MalformedRewrite> = Vec::new();
        for (i, cone) in self.cones().iter().enumerate() {
            if let Err(e) = cone.check(recursive) {
                errors.push(MalformedRewrite::Cone(i, e));
            }

            // Check that the cone is not trivial.
            if cone.is_identity() {
                errors.push(MalformedRewrite::TrivialCone(i));
            }
        }

        // Check that the cones are ordered by index.
        if self.cones().windows(2).any(|w| w[0].index > w[1].index) {
            errors.push(MalformedRewrite::NotOrderedCorrectly);
        }

        REWRITE_CACHE.with_borrow_mut(|cache| cache.insert(self.clone(), errors.clone()));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Cone {
    pub fn check(&self, recursive: bool) -> Result<(), Vec<MalformedCone>> {
        let mut errors = vec![];

        if recursive {
            // Check that the source is well-formed.
            for (i, cs) in self.source().iter().enumerate() {
                if let Err(e) = cs.forward.check_worker(recursive) {
                    errors.push(MalformedCone::Source(i, e));
                }
                if let Err(e) = cs.backward.check_worker(recursive) {
                    errors.push(MalformedCone::Source(i, e));
                }
            }

            // Check that the target is well-formed.
            if let Err(e) = self.target().forward.check_worker(recursive) {
                errors.push(MalformedCone::Target(e));
            }
            if let Err(e) = self.target().backward.check_worker(recursive) {
                errors.push(MalformedCone::Target(e));
            }

            // Check that the regular slices are well-formed.
            for (i, slice) in self.regular_slices().iter().enumerate() {
                if let Err(e) = slice.check_worker(recursive) {
                    errors.push(MalformedCone::RegularSlice(i, e));
                }
            }

            // Check that the singular slices are well-formed.
            for (i, slice) in self.singular_slices().iter().enumerate() {
                if let Err(e) = slice.check_worker(recursive) {
                    errors.push(MalformedCone::SingularSlice(i, e));
                }
            }
        }

        // Check commutativity conditions.
        if !self
            .regular_slices()
            .first()
            .unwrap()
            .equivalent(&self.target().forward)
        {
            errors.push(MalformedCone::NotCommutative(0));
        }

        for (i, (cs, singular_slice)) in
            std::iter::zip(self.source(), self.singular_slices()).enumerate()
        {
            match cs.forward.compose(singular_slice) {
                Ok(f) if f.equivalent(&self.regular_slices()[i]) => { /* no error */ }
                Ok(_) => errors.push(MalformedCone::NotCommutative(i)),
                Err(ce) => errors.push(ce.into()),
            }

            match cs.backward.compose(singular_slice) {
                Ok(f) if f.equivalent(&self.regular_slices()[i + 1]) => { /* no error */ }
                Ok(_) => errors.push(MalformedCone::NotCommutative(i + 1)),
                Err(ce) => errors.push(ce.into()),
            }
        }

        if !self
            .regular_slices()
            .last()
            .unwrap()
            .equivalent(&self.target().backward)
        {
            errors.push(MalformedCone::NotCommutative(self.len()));
        }

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
    #[error("cone {0} is malformed: {1:?}")]
    Cone(usize, Vec<MalformedCone>),

    #[error("cone {0} is trivial.")]
    TrivialCone(usize),

    #[error("cones are not ordered correctly.")]
    NotOrderedCorrectly,
}

#[derive(Clone, Debug, Error)]
pub enum MalformedCone {
    #[error(transparent)]
    Composition(#[from] CompositionError),

    #[error("source {0} is malformed: {1:?}")]
    Source(usize, Vec<MalformedRewrite>),

    #[error("target is malformed: {0:?}")]
    Target(Vec<MalformedRewrite>),

    #[error("regular slice {0} is malformed: {1:?}")]
    RegularSlice(usize, Vec<MalformedRewrite>),

    #[error("singular slice {0} is malformed: {1:?}")]
    SingularSlice(usize, Vec<MalformedRewrite>),

    #[error("cone fails to be commutative at regular height {0}.")]
    NotCommutative(usize),
}

impl Rewrite {
    #[must_use]
    pub fn equivalent(&self, other: &Self) -> bool {
        use Rewrite::{Rewrite0, RewriteN};
        match (self, other) {
            (Rewrite0(f), Rewrite0(g)) => f.equivalent(g),
            (RewriteN(f), RewriteN(g)) => f.equivalent(g),
            (_, _) => false,
        }
    }
}

impl Rewrite0 {
    #[must_use]
    pub fn equivalent(&self, other: &Self) -> bool {
        self.boundaries() == other.boundaries()
    }
}

impl RewriteN {
    #[must_use]
    pub fn equivalent(&self, other: &Self) -> bool {
        // Do all the cheap and non-recursive tests first
        self.dimension() == other.dimension()
            && self.cones().len() == other.cones().len()
            // Actually do the recursion only if really needed
            && self
                .cones()
                .iter()
                .zip(other.cones().iter())
                .all(|(sc, oc)| sc.index == oc.index && sc.equivalent(oc))
    }
}

impl Cone {
    #[must_use]
    pub fn equivalent(&self, other: &Self) -> bool {
        self.source() == other.source()
            && self.target() == other.target()
            && self
                .regular_slices()
                .iter()
                .zip(other.regular_slices().iter())
                .all(|(f, g)| f.equivalent(g))
            && self
                .singular_slices()
                .iter()
                .zip(other.singular_slices().iter())
                .all(|(f, g)| f.equivalent(g))
    }
}
