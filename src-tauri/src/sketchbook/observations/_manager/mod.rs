use crate::sketchbook::ids::DatasetId;
use crate::sketchbook::observations::Dataset;
use crate::sketchbook::Manager;
use std::collections::HashMap;

/// **(internal)** Functionality for exporting datasets to files.
mod _impl_export_dataset;
/// **(internal)** Implementation of the safe identifier generating.
mod _impl_id_generating;
/// **(internal)** Functionality for loading datasets from files.
mod _impl_load_dataset;
/// **(internal)** Basic utility methods for `ObservationManager`.
mod _impl_manager;
/// **(internal)** Implementation of event-based API for the [crate::app::state::SessionState] trait.
mod _impl_session_state;

/// Class to manage all observations and datasets.
///
/// `ObservationManager` can be managed through its classical Rust API, as well as
/// through the external events (as it implements the `SessionState` event).
#[derive(Clone, Debug, PartialEq)]
pub struct ObservationManager {
    datasets: HashMap<DatasetId, Dataset>,
}

impl Manager for ObservationManager {}

impl Default for ObservationManager {
    /// Default manager instance with no datasets.
    fn default() -> ObservationManager {
        ObservationManager::new_empty()
    }
}
