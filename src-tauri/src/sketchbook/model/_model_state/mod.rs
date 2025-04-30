use crate::sketchbook::ids::{LayoutId, UninterpretedFnId, VarId};
use crate::sketchbook::layout::Layout;
use crate::sketchbook::model::{Regulation, UninterpretedFn, UpdateFn, Variable};
use crate::sketchbook::Manager;
use std::collections::{HashMap, HashSet};

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
/// **(internal)** Implementation of event-based API for the [crate::app::state::SessionState] trait.
mod _impl_session_state;

/// Structure representing the state of the "model" part of the sketch. `ModelState`
/// encompasses information about the regulatory network and the PSBN. Specifically, it
/// covers:
/// - set of Boolean variables
/// - set of regulations between the variables
/// - set of function symbols (or uninterpreted functions)
/// - Boolean update functions for each variable
/// - layout information regarding the regulatory network
///
/// `ModelState` can be observed/edited using its classical Rust API, as well as through
/// the external events (as it implements the `SessionState` event).
#[derive(Clone, Debug, PartialEq)]
pub struct ModelState {
    variables: HashMap<VarId, Variable>,
    regulations: HashSet<Regulation>,
    update_fns: HashMap<VarId, UpdateFn>,
    uninterpreted_fns: HashMap<UninterpretedFnId, UninterpretedFn>,
    layouts: HashMap<LayoutId, Layout>,
}

impl Manager for ModelState {}

impl Default for ModelState {
    /// Default model object with no Variables, Uninterpreted Functions, or Regulations yet.
    /// It contains a single empty default Layout.
    fn default() -> ModelState {
        ModelState::new_empty()
    }
}
