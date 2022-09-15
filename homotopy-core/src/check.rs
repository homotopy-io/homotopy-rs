use std::cell::RefCell;

use homotopy_common::hash::FastHashMap;
use thiserror::Error;

use crate::{
    common::Mode,
    diagram::RewritingError,
    rewrite::{CompositionError, Cone},
    Diagram, DiagramN, Direction, Height, Rewrite, Rewrite0, RewriteN,
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
        if let Some(errors) = DIAGRAM_CACHE.with(|cache| cache.borrow().get(self).cloned()) {
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
        if let Some(errors) = REWRITE_CACHE.with(|cache| cache.borrow().get(self).cloned()) {
            return if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            };
        }

        let mut errors: Vec<MalformedRewrite> = Vec::new();
        for (i, cone) in self.cones().iter().enumerate() {
            if let Err(e) = cone.check(mode) {
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

        REWRITE_CACHE.with(|cache| cache.borrow_mut().insert(self.clone(), errors.clone()));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Cone {
    pub fn check(&self, mode: Mode) -> Result<(), Vec<MalformedCone>> {
        let mut errors = vec![];

        if mode == Mode::Deep {
            // Check that the source is well-formed.
            for (i, cs) in self.source().iter().enumerate() {
                if let Err(e) = cs.forward.check_worker(mode) {
                    errors.push(MalformedCone::Source(i, e));
                }
                if let Err(e) = cs.backward.check_worker(mode) {
                    errors.push(MalformedCone::Source(i, e));
                }
            }

            // Check that the target is well-formed.
            if let Err(e) = self.target().forward.check_worker(mode) {
                errors.push(MalformedCone::Target(e));
            }
            if let Err(e) = self.target().backward.check_worker(mode) {
                errors.push(MalformedCone::Target(e));
            }

            // Check that the regular slices are well-formed.
            for (i, slice) in self.regular_slices().iter().enumerate() {
                if let Err(e) = slice.check_worker(mode) {
                    errors.push(MalformedCone::RegularSlice(i, e));
                }
            }

            // Check that the singular slices are well-formed.
            for (i, slice) in self.singular_slices().iter().enumerate() {
                if let Err(e) = slice.check_worker(mode) {
                    errors.push(MalformedCone::SingularSlice(i, e));
                }
            }
        }

        // Check commutativity conditions.
        if self.regular_slices().first().unwrap().strip_labels()
            != self.target().forward.strip_labels()
        {
            errors.push(MalformedCone::NotCommutative(0));
        }

        for (i, (cs, singular_slice)) in
            std::iter::zip(self.source(), self.singular_slices()).enumerate()
        {
            match cs
                .forward
                .strip_labels()
                .compose(&singular_slice.strip_labels())
            {
                Ok(f) if f == self.regular_slices()[i].strip_labels() => { /* no error */ }
                Ok(_) => errors.push(MalformedCone::NotCommutative(i)),
                Err(ce) => errors.push(ce.into()),
            }

            match cs
                .backward
                .strip_labels()
                .compose(&singular_slice.strip_labels())
            {
                Ok(f) if f == self.regular_slices()[i + 1].strip_labels() => { /* no error */ }
                Ok(_) => errors.push(MalformedCone::NotCommutative(i + 1)),
                Err(ce) => errors.push(ce.into()),
            }
        }

        if self.regular_slices().last().unwrap().strip_labels()
            != self.target().backward.strip_labels()
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
    pub fn strip_labels(&self) -> Self {
        use Rewrite::{Rewrite0, RewriteN};
        match self {
            Rewrite0(f) => Rewrite0(f.strip_labels()),
            RewriteN(f) => RewriteN(f.strip_labels()),
        }
    }
}

impl Rewrite0 {
    pub fn strip_labels(&self) -> Self {
        Self(self.0.as_ref().map(|(s, t, _)| (*s, *t, None)))
    }
}

impl RewriteN {
    pub fn strip_labels(&self) -> Self {
        RewriteN::new_unsafe(
            self.dimension(),
            self.cones()
                .iter()
                .map(|c| {
                    Cone::new(
                        c.index,
                        c.source()
                            .iter()
                            .map(|cs| cs.map(Rewrite::strip_labels))
                            .collect(),
                        c.target().map(Rewrite::strip_labels),
                        c.regular_slices()
                            .iter()
                            .map(Rewrite::strip_labels)
                            .collect(),
                        c.singular_slices()
                            .iter()
                            .map(Rewrite::strip_labels)
                            .collect(),
                    )
                })
                .collect(),
        )
    }
}
