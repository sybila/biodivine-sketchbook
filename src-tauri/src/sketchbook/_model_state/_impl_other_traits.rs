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
/// TODO: do we want to consider layouts in a way that is implemented right now?
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

    fn from_str(s: &str) -> Result<ModelState, <ModelState as FromStr>::Err> {
        match serde_json::from_str(s) {
            Ok(reg_state) => Ok(reg_state),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Display for ModelState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
