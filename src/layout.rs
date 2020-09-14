use crate::diagram::*;
use crate::common::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point<T>(pub T, pub T);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Average<T>(Vec<T>, T);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Distance<T>(T, T);

fn make_average_constraints(diagram: &DiagramN) -> Vec<Average<Point<Height>>> {
    let mut constraints = Vec::new();
    let cospans = diagram.cospans();
    let slices = diagram.slices();

    for i in 0..diagram.size() {
        let cospan = &cospans[i];
        let forward = cospan.forward.to_n().unwrap();
        let backward = cospan.backward.to_n().unwrap();
        let regular0 = &slices[i * 2].to_n().unwrap();
        let singular = &slices[i * 2 + 1].to_n().unwrap();
        let regular1 = &slices[i * 2 + 2].to_n().unwrap();

        constraints.extend((0..singular.size()).map(|x| {
            Average(
                forward
                    .singular_preimage(x)
                    .map(|xp| Point(Height::Singular(xp), Height::Regular(i)))
                    .collect(),
                Point(Height::Singular(x), Height::Singular(i)),
            )
        }));

        constraints.extend((0..singular.size()).map(|x| {
            Average(
                backward
                    .singular_preimage(x)
                    .map(|xp| Point(Height::Singular(xp), Height::Regular(i + 1)))
                    .collect(),
                Point(Height::Singular(x), Height::Singular(i)),
            )
        }));

        constraints.extend((0..regular0.size() + 1).map(|x| {
            Average(
                forward
                    .regular_preimage(x)
                    .map(|xp| Point(Height::Regular(xp), Height::Singular(i)))
                    .collect(),
                Point(Height::Regular(x), Height::Regular(i)),
            )
        }));

        constraints.extend((0..regular1.size() + 1).map(|x| {
            Average(
                backward
                    .regular_preimage(x)
                    .map(|xp| Point(Height::Regular(xp), Height::Singular(i)))
                    .collect(),
                Point(Height::Regular(x), Height::Regular(i + 1)),
            )
        }));
    }

    constraints
}

fn make_distance_constraints(diagram: &DiagramN) -> Vec<Distance<Point<Height>>> {
    let mut constraints = Vec::new();

    for (y, slice) in diagram.slices().into_iter().enumerate() {
        let slice = slice.to_n().unwrap();
        let y = Height::from_int(y);

        for x in 0..slice.size() {
            constraints.push(Distance(
                Point(Height::Regular(x), y),
                Point(Height::Singular(x), y),
            ));
            constraints.push(Distance(
                Point(Height::Singular(x), y),
                Point(Height::Regular(x + 1), y),
            ));
        }
    }

    constraints
}

#[derive(Clone, Copy)]
enum Var {
    Link(Point<Height>),
    Index(usize),
}

struct State {
    variables: HashMap<Point<Height>, Var>,
    positions: Vec<f32>,
}

impl State {
    fn new() -> Self {
        State {
            variables: HashMap::new(),
            positions: Vec::new(),
        }
    }

    fn follow(&self, point: Point<Height>) -> Point<Height> {
        match self.variables.get(&point) {
            Some(Var::Link(target)) => *target,
            _ => point,
        }
    }

    fn position(&self, point: Point<Height>) -> f32 {
        match self.variables.get(&point) {
            Some(Var::Link(target)) => self.position(*target),
            Some(Var::Index(index)) => self.positions[*index],
            None => panic!(),
        }
    }

    fn link(&mut self, p: Point<Height>, q: Point<Height>) {
        if p < q {
            self.variables.insert(q, Var::Link(self.follow(p)));
        } else {
            self.variables.insert(p, Var::Link(self.follow(q)));
        }
    }

    fn add_var(&mut self, point: Point<Height>) -> usize {
        match self.variables.get(&point).cloned() {
            Some(Var::Link(p)) => self.add_var(p),
            Some(Var::Index(index)) => index,
            None => {
                let index = self.positions.len();
                self.variables.insert(point, Var::Index(index));
                self.positions.push(0.0);
                index
            }
        }
    }

    fn step_average(&mut self, constraint: &Average<usize>) -> bool {
        let Average(xs, y) = constraint;
        let xs_value = xs.iter().map(|x| self.positions[*x]).sum::<f32>() / (xs.len() as f32);
        let y_value = self.positions[*y];
        let diff = xs_value - y_value;

        if diff > 0.01 {
            self.positions[*y] += diff;
            true
        } else if diff < -0.01 {
            for x in xs {
                self.positions[*x] -= diff;
            }
            true
        } else {
            false
        }
    }

    fn step_distance(&mut self, constraint: &Distance<usize>) -> bool {
        let Distance(x, y) = constraint;
        let x_value = self.positions[*x];
        let y_value = self.positions[*y];

        if y_value + 0.01 < x_value + 1.0 {
            self.positions[*y] = x_value + 1.0;
            true
        } else {
            false
        }
    }

    fn finish(self) -> HashMap<Point<Height>, f32> {
        self.variables
            .keys()
            .map(|p| (*p, self.position(*p)))
            .collect()
    }
}

pub fn layout(diagram: &DiagramN) -> HashMap<Point<Height>, f32> {
    let average = make_average_constraints(diagram);
    let distance = make_distance_constraints(diagram);

    // An average constraint between exactly two variables should link them.
    let mut state = State::new();

    let average = average
        .into_iter()
        .filter(|Average(xs, y)| {
            if xs.len() == 1 {
                state.link(xs[0], *y);
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    // Add points to state, generating the variable ids.
    let average = average
        .into_iter()
        .map(|Average(xs, y)| {
            let xs = xs.into_iter().map(|x| state.add_var(x)).collect();
            let y = state.add_var(y);
            Average(xs, y)
        })
        .collect::<Vec<_>>();

    let mut distance_seen = HashSet::new();

    let distance = distance
        .into_iter()
        .map(|Distance(x, y)| Distance(state.add_var(x), state.add_var(y)))
        .filter(|c| {
            if distance_seen.contains(c) {
                false
            } else {
                distance_seen.insert(c.clone());
                true
            }
        })
        .collect::<Vec<_>>();

    // Run constraints
    for _ in 0..1000 {
        let mut changed = false;

        for c in &distance {
            changed = state.step_distance(c) || changed;
        }

        for c in &average {
            changed = state.step_average(c) || changed;
        }

        if !changed {
            break;
        }
    }

    state.finish()
}
