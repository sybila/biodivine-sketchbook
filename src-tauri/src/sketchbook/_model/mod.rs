use crate::sketchbook::layout::Layout;
use crate::sketchbook::{
    LayoutId, Regulation, UninterpretedFn, UninterpretedFnId, UpdateFn, VarId, Variable,
};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// **(internal)** Methods for converting between `ModelState` and `BooleanNetwork`.
mod _impl_convert_bn;
/// **(internal)** Methods for converting between `ModelState` and `RegulatoryGraph`.
mod _impl_convert_reg_graph;
/// **(internal)** Methods for safely constructing or editing instances of `ModelState`.
mod _impl_editing;
/// **(internal)** Implementation of the safe identifier generating.
mod _impl_id_generating;
/// **(internal)** Methods for observing instances of `ModelState` (various getters, etc.).
mod _impl_observing;
/// **(internal)** Implementation of serialization traits [Serialize] and [Deserialize].
mod _impl_serde;
/// **(internal)** Implementation of event-based API for the [SessionState] trait.
mod _impl_session_state;

/// Object representing the state of the model in the Boolean network editor. The model encompasses variables,
/// regulations, uninterpreted functions, update functions, and layout information.
///
/// `ModelState` can be observed/edited using its classical Rust API, as well as through the
/// external events (as it implements the `SessionState` event).
#[derive(Clone, Debug, PartialEq)]
pub struct ModelState {
    variables: HashMap<VarId, Variable>,
    regulations: HashSet<Regulation>,
    update_fns: HashMap<VarId, UpdateFn>,
    uninterpreted_fns: HashMap<UninterpretedFnId, UninterpretedFn>,
    layouts: HashMap<LayoutId, Layout>,
    placeholder_variables: HashSet<VarId>,
}

impl Default for ModelState {
    /// Default model object with no Variables, Uninterpreted Functions, or Regulations yet.
    /// It contains a single empty default Layout
    fn default() -> ModelState {
        ModelState::new()
    }
}

impl FromStr for ModelState {
    type Err = String;

    /// Use json de-serialization to construct `ModelState` from string.
    /// See [_impl_serde] for details.
    fn from_str(s: &str) -> Result<ModelState, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}

impl Display for ModelState {
    /// Use json serialization to convert `ModelState` to string.
    /// See [_impl_serde] for details.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
