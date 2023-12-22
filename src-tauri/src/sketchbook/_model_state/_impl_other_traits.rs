use crate::sketchbook::ModelState;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

impl Default for ModelState {
    fn default() -> ModelState {
        ModelState::new()
    }
}

/// To consider two `ModelStates` equivalent, we generally assume that they have the same
/// number of variables, with the same ids and names. Furthermore, they also need to have the
/// same regulations and layouts. The order of the variables and regulations does not matter.
impl PartialEq for ModelState {
    fn eq(&self, other: &ModelState) -> bool {
        self.variables == other.variables
            && self.regulations == other.regulations
            && self.layouts == other.layouts
    }
}

impl Eq for ModelState {}

impl FromStr for ModelState {
    type Err = String;

    /// Use json de-serialization to construct `ModelState` from string.
    fn from_str(s: &str) -> Result<ModelState, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}

impl Display for ModelState {
    /// Use json serialization to convert `ModelState` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
