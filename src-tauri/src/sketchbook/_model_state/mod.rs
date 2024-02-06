use crate::sketchbook::layout::{Layout, LayoutId};
use crate::sketchbook::{Regulation, VarId, Variable, ParamId, Parameter};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// **(internal)** Methods for safely constructing or editing instances of `ModelState`.
mod _impl_editing;
/// **(internal)** Implementation of the safe identifier generating.
mod _impl_id_generating;
/// **(internal)** Methods for observing instances of `ModelState` (various getters, etc.).
mod _impl_observing;
/// **(internal)** Methods for converting between `ModelState` and `RegulatoryGraph`.
mod _impl_reg_graph_conversion;
/// **(internal)** Implementation of serialization traits [Serialize] and [Deserialize].
mod _impl_serde;
/// **(internal)** Implementation of the [SessionState] trait.
mod _impl_session_state;

/// Object representing the state of the model in the Regulations editor, which includes variables,
/// regulations, and layout information.
///
/// `ModelState` can be observed/edited using its classical Rust API, as well as through the
/// external events (as it implements the `SessionState` event).
#[derive(Clone, Debug, PartialEq)]
pub struct ModelState {
    variables: HashMap<VarId, Variable>,
    parameters: HashMap<ParamId, Parameter>,
    regulations: HashSet<Regulation>,
    layouts: HashMap<LayoutId, Layout>,
}

impl Default for ModelState {
    fn default() -> ModelState {
        ModelState::new()
    }
}

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
