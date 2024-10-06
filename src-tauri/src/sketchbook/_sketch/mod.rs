use crate::sketchbook::model::ModelState;
use crate::sketchbook::observations::ObservationManager;
use crate::sketchbook::properties::PropertyManager;
use crate::sketchbook::Manager;

/// **(internal)** Utilities to check consistency of `Sketch` instances.
mod _impl_consistency;
/// **(internal)** Exporting sketch in various formats.
mod _impl_export;
/// **(internal)** Importing sketch in various formats.
mod _impl_import;
/// **(internal)** Implementation of event-based API for the [SessionState] trait.
mod _impl_session_state;
/// **(internal)** Utility methods for `Sketch`.
mod _impl_sketch;

/// Object encompassing all of the individual modules of the Boolean network sketch.
///
/// Most of the actual functionality is implemented by the modules themselves, `Sketch`
/// currently only distributes events and handles situations when cooperation between
/// modules is needed.
#[derive(Clone, Debug, PartialEq)]
pub struct Sketch {
    pub model: ModelState,
    pub observations: ObservationManager,
    pub properties: PropertyManager,
    pub annotation: String,
}

impl Manager for Sketch {}

impl Default for Sketch {
    /// Default empty sketch.
    fn default() -> Sketch {
        Sketch {
            model: ModelState::default(),
            observations: ObservationManager::default(),
            properties: PropertyManager::default(),
            annotation: String::default(),
        }
    }
}
