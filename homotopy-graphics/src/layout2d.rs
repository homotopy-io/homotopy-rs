use crate::geometry::Point;
use homotopy_core::common::{Boundary, Height, SliceIndex};
use homotopy_core::diagram::DiagramN;
use homotopy_core::rewrite::RewriteN;
use serde::Serialize;
use std::convert::{Into, TryFrom, TryInto};
use thiserror::Error;

#[derive(Debug)]
struct Constraint(Vec<(usize, usize)>, (usize, usize));

impl Constraint {
    fn build(diagram: &DiagramN) -> Vec<Self> {
        use Height::{Regular, Singular};

        let mut constraints = Vec::new();
        let cospans = diagram.cospans();
        let slices: Vec<_> = diagram.slices().collect();

        for i in 0..diagram.size() {
            let cospan = &cospans[i];
            let forward: &RewriteN = (&cospan.forward).try_into().unwrap();
            let backward: &RewriteN = (&cospan.backward).try_into().unwrap();
            let regular0: &DiagramN = (&slices[i * 2]).try_into().unwrap();
            let singular: &DiagramN = (&slices[i * 2 + 1]).try_into().unwrap();
            let regular1: &DiagramN = (&slices[i * 2 + 2]).try_into().unwrap();

            constraints.extend((0..singular.size()).map(|x| {
                Self(
                    forward
                        .singular_preimage(x)
                        .map(|xp| (Singular(xp).to_int(), Regular(i).to_int()))
                        .collect(),
                    (Singular(x).to_int(), Singular(i).to_int()),
                )
            }));

            constraints.extend((0..singular.size()).map(|x| {
                Self(
                    backward
                        .singular_preimage(x)
                        .map(|xp| (Singular(xp).to_int(), Regular(i + 1).to_int()))
                        .collect(),
                    (Singular(x).to_int(), Singular(i).to_int()),
                )
            }));

            constraints.extend((0..=regular0.size()).map(|x| {
                Self(
                    forward
                        .regular_preimage(x)
                        .map(|xp| (Regular(xp).to_int(), Singular(i).to_int()))
                        .collect(),
                    (Regular(x).to_int(), Regular(i).to_int()),
                )
            }));

            constraints.extend((0..=regular1.size()).map(|x| {
                Self(
                    backward
                        .regular_preimage(x)
                        .map(|xp| (Regular(xp).to_int(), Singular(i).to_int()))
                        .collect(),
                    (Regular(x).to_int(), Regular(i + 1).to_int()),
                )
            }));
        }

        constraints
    }
}

/// Position store used in the [Solver].
struct Positions(Vec<Vec<f32>>);

impl Positions {
    fn get(&self, point: (usize, usize)) -> f32 {
        self.0[point.1][point.0]
    }

    fn set(&mut self, point: (usize, usize), value: f32) {
        self.0[point.1][point.0] = value;
    }

    fn propagate(&mut self, constraint: &Constraint) -> bool {
        let source_sum = constraint.0.iter().map(|p| self.get(*p)).sum::<f32>();

        let target_pos = self.get(constraint.1);

        let diff = source_sum / (constraint.0.len() as f32) - target_pos;

        if diff > 0.01 {
            self.set(constraint.1, self.get(constraint.1) + diff);
            true
        } else if diff < -0.01 {
            constraint
                .0
                .iter()
                .for_each(|p| self.set(*p, self.get(*p) - diff));
            true
        } else {
            false
        }
    }

    fn width(&self) -> f32 {
        self.0
            .iter()
            .map(|row| row.last().unwrap())
            .max_by_key(|x| (**x * 100.0).floor() as usize)
            .copied()
            .unwrap()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("layout solver expected a diagram of dimension two or higher.")]
    Dimension,
}

/// Iterative solver for 2-dimensional diagram layouts.
pub struct Solver {
    positions: Positions,
    constraints: Vec<Constraint>,
}

impl Solver {
    pub fn new(diagram: &DiagramN) -> Result<Self, Error> {
        if diagram.dimension() < 2 {
            return Err(Error::Dimension);
        }

        let positions = Positions(
            diagram
                .slices()
                .map(|slice| {
                    (0..DiagramN::try_from(slice).unwrap().size() * 2 + 1)
                        .map(|i| i as f32)
                        .collect()
                })
                .collect(),
        );

        let constraints: Vec<_> = Constraint::build(&diagram)
            .into_iter()
            .filter(|c| !c.0.is_empty())
            .collect();

        Ok(Self {
            positions,
            constraints,
        })
    }

    fn propagate(&mut self) -> bool {
        let mut changed = false;

        for constraint in &self.constraints {
            changed = self.positions.propagate(constraint) || changed;
        }

        changed
    }

    fn distance(&mut self) -> bool {
        let mut changed = false;

        for y in 0..self.positions.0.len() {
            for x in 1..self.positions.0[y].len() {
                let first = self.positions.0[y][x - 1];
                let second = self.positions.0[y][x];

                if second < first + 1.0 {
                    self.positions.0[y][x] = first + 1.0;
                    changed = true;
                }
            }
        }

        changed
    }

    pub fn step(&mut self) -> bool {
        let mut changed = false;
        changed = self.distance() || changed;
        changed = self.propagate() || changed;
        changed
    }

    /// Run as many iterations of the solver until the positions stabilize or a maximum number of
    /// steps is reached. Then return the number of steps.
    pub fn solve(&mut self, max_steps: usize) -> usize {
        for step in 0..max_steps {
            if !self.step() {
                return step;
            }
        }

        max_steps
    }

    fn finish(self) -> Layout {
        Layout {
            width: self.positions.width(),
            positions: self.positions.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Layout {
    positions: Vec<Vec<f32>>,
    width: f32,
}

impl Layout {
    /// Solve for a diagram's layout with a given maximum number of iterations.
    ///
    /// Expects diagrams of dimension two or higher.
    pub fn new(diagram: &DiagramN, max_steps: usize) -> Result<Self, Error> {
        let mut solver = Solver::new(diagram)?;
        solver.solve(max_steps);
        Ok(solver.finish())
    }

    /// The position of a logical point in the layout.
    pub fn get(&self, x: SliceIndex, y: SliceIndex) -> Option<Point> {
        let positions = &self.positions;

        let slice = match y {
            SliceIndex::Boundary(Boundary::Source) => positions.first()?,
            SliceIndex::Boundary(Boundary::Target) => positions.last()?,
            SliceIndex::Interior(height) => positions.get(height.to_int())?,
        };

        let y_pos = match y {
            SliceIndex::Boundary(Boundary::Source) => 0.0,
            SliceIndex::Boundary(Boundary::Target) => positions.len() as f32 + 1.0,
            SliceIndex::Interior(height) => height.to_int() as f32 + 1.0,
        };

        let x_pos = match x {
            SliceIndex::Boundary(Boundary::Source) => 0.0,
            SliceIndex::Boundary(Boundary::Target) => self.width + 2.0,
            SliceIndex::Interior(height) => slice.get(height.to_int())? + 1.0,
        };

        Some((x_pos, y_pos).into())
    }
}
