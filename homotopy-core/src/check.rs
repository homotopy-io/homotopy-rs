use std::cell::RefCell;

use homotopy_common::hash::FastHashMap;
use thiserror::Error;

use crate::{
    common::Mode, diagram::RewritingError, rewrite::CompositionError, Diagram, DiagramN, Direction,
    Height, Rewrite, RewriteN,
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
        for cone in self.cones() {
            if cone.len() == 0 {
                if cone.internal.target.forward != cone.internal.target.backward {
                    errors.push(MalformedRewrite::NotSingularity(cone.index));
                }
            } else {
                // Check that the subslices are well-formed.
                if mode == Mode::Deep {
                    for (i, slice) in cone.internal.slices.iter().enumerate() {
                        if let Err(e) = slice.check_worker(mode) {
                            errors.push(MalformedRewrite::Slice(i, e));
                        }
                    }
                }

                // Check that the squares commute.
                let len = cone.len();

                match cone.internal.source[0]
                    .forward
                    .compose(&cone.internal.slices[0])
                {
                    Ok(f) if f == cone.internal.target.forward => { /* no error */ }
                    Ok(_) => errors.push(MalformedRewrite::NotCommutativeLeft(cone.index)),
                    Err(ce) => errors.push(ce.into()),
                };

                for i in 0..len - 1 {
                    let f = cone.internal.source[i]
                        .backward
                        .compose(&cone.internal.slices[i]);
                    let g = cone.internal.source[i + 1]
                        .forward
                        .compose(&cone.internal.slices[i + 1]);
                    match (f, g) {
                        (Ok(f), Ok(g)) if f == g => { /* no error */ }
                        (Ok(_), Ok(_)) => errors.push(MalformedRewrite::NotCommutativeMiddle(
                            cone.index + i,
                            cone.index + i + 1,
                        )),
                        (Ok(_), Err(ce)) | (Err(ce), Ok(_)) => errors.push(ce.into()),
                        (Err(f_ce), Err(g_ce)) => {
                            errors.push(f_ce.into());
                            errors.push(g_ce.into());
                        }
                    }
                }

                match cone.internal.source[len - 1]
                    .backward
                    .compose(&cone.internal.slices[len - 1])
                {
                    Ok(f) if f == cone.internal.target.backward => { /* no error */ }
                    Ok(_) => errors.push(MalformedRewrite::NotCommutativeRight(len - 1)),
                    Err(ce) => errors.push(ce.into()),
                };
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

    #[error("slice {0:?} is malformed: {1:?}")]
    Slice(usize, Vec<MalformedRewrite>),

    #[error("slice {0} of target cannot be a singularity.")]
    NotSingularity(usize),

    #[error("square to the left of slice {0} does not commute.")]
    NotCommutativeLeft(usize),

    #[error("square to the right of slice {0} does not commute.")]
    NotCommutativeRight(usize),

    #[error("square between slices {0} and {1} does not commute.")]
    NotCommutativeMiddle(usize, usize),
}
