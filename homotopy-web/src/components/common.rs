use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Visible,
    Hidden,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "visibility: {}",
            match self {
                Self::Visible => "visible",
                Self::Hidden => "hidden",
            }
        )
    }
}

impl From<bool> for Visibility {
    fn from(b: bool) -> Self {
        if b {
            Self::Visible
        } else {
            Self::Hidden
        }
    }
}
