use crate::sketchbook::ids::PerturbationId;
use crate::sketchbook::perturbations::Perturbation;
use crate::sketchbook::Manager;
use std::collections::HashMap;

/// **(internal)** Implementation of the safe identifier generating.
mod _impl_id_generating;
/// **(internal)** Basic utility methods for `PerturbationManager`.
mod _impl_manager;
/// **(internal)** Implementation of event-based API for the [crate::app::state::SessionState] trait.
mod _impl_session_state;

/// Class to manage all properties of the sketch.
///
/// `PerturbationManager` can be managed through its classical Rust API, as well as
/// through the external events (as it implements the `SessionState` trait).
#[derive(Clone, Debug, PartialEq)]
pub struct PerturbationManager {
    perturbations: HashMap<PerturbationId, Perturbation>,
}

impl Manager for PerturbationManager {}

impl Default for PerturbationManager {
    /// Default manager instance with no perturbations.
    fn default() -> PerturbationManager {
        PerturbationManager::new_empty()
    }
}
