use crate::sketchbook::model::{Essentiality, Monotonicity};
use serde::{Deserialize, Serialize};

/// Data regarding properties of a particular argument of an uninterpreted function.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FnArgumentProperty {
    pub essential: Essentiality,
    pub monotonicity: Monotonicity,
}

impl FnArgumentProperty {
    /// New `FnArgument` with given monotonicity and essentiality.
    pub fn new(essential: Essentiality, monotonicity: Monotonicity) -> FnArgumentProperty {
        FnArgumentProperty {
            essential,
            monotonicity,
        }
    }
}

impl Default for FnArgumentProperty {
    /// Default `FnArgument` with unknown monotonicity and essentiality..
    fn default() -> FnArgumentProperty {
        FnArgumentProperty {
            essential: Essentiality::Unknown,
            monotonicity: Monotonicity::Unknown,
        }
    }
}
