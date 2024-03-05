use std::{iter::Product, ops::Mul};

use homotopy_core::Direction;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Parity {
    Even,
    Odd,
}

impl From<bool> for Parity {
    fn from(is_even: bool) -> Self {
        if is_even {
            Self::Even
        } else {
            Self::Odd
        }
    }
}

impl Parity {
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::Even => Self::Odd,
            Self::Odd => Self::Even,
        }
    }

    #[must_use]
    pub const fn is_even(self) -> bool {
        match self {
            Self::Even => true,
            Self::Odd => false,
        }
    }

    #[must_use]
    pub const fn from_direction(direction: Direction) -> Self {
        match direction {
            Direction::Forward => Self::Even,
            Direction::Backward => Self::Odd,
        }
    }

    #[must_use]
    pub fn from_orientation(orientation: &[(usize, Direction)]) -> Self {
        orientation
            .iter()
            .map(|(_, direction)| Self::from_direction(*direction))
            .product()
    }
}

// Parity composition

impl Mul for Parity {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from(self == rhs)
    }
}

impl Product for Parity {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::Even, |a, b| a * b)
    }
}
