use crate::sketchbook::{Essentiality, Monotonicity};
use serde::{Deserialize, Serialize};

/// Data regarding an argument of an uninterpreted function.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FnArgument {
    pub essential: Essentiality,
    pub monotonicity: Monotonicity,
}

impl FnArgument {
    /// New `FnArgument` with given monotonicity and essentiality..
    pub fn new(essential: Essentiality, monotonicity: Monotonicity) -> FnArgument {
        FnArgument {
            essential,
            monotonicity,
        }
    }
}

impl Default for FnArgument {
    /// Default `FnArgument` with unknown monotonicity and essentiality..
    fn default() -> FnArgument {
        FnArgument {
            essential: Essentiality::Unknown,
            monotonicity: Monotonicity::Unknown,
        }
    }
}
