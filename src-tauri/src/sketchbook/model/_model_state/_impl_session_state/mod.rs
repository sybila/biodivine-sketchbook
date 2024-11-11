use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::model::ModelState;

/// **(internal)** Implementation for events related to `layouts` of the model.
pub mod _events_layout;
/// **(internal)** Implementation for events related to `regulations` of the model.
pub mod _events_regulations;
/// **(internal)** Implementation for events related to `uninterpreted functions` of the model.
pub mod _events_uninterpreted_fns;
/// **(internal)** Implementation for events related to `variables` of the model and
/// their `update functions`.
pub mod _events_variables;
/// **(internal)** Implementation for `refresh` (getter) events.
pub mod _refresh_events;

/* Constants for event path segments in `ModelState` for event handling. */

// events being delegated to `variables` subcomponent
const VAR_EVENT_PATH: &str = "variable";
// events being delegated to `uninterpreted fns` subcomponent
const FN_EVENT_PATH: &str = "uninterpreted_fn";
// events being delegated to `regulations` subcomponent
const REGULATION_EVENT_PATH: &str = "regulation";
// events being delegated to `layouts` subcomponent
const LAYOUT_EVENT_PATH: &str = "layout";

/* Constants for refresh event path segments in `ModelState` for retrieving data. */

// refresh all model components at once
const REFRESH_MODEL_PATH: &str = "get_whole_model";
// refresh all model variables
const REFRESH_VARS_PATH: &str = "get_variables";
// refresh all model uninterpreted fns
const REFRESH_FNS_PATH: &str = "get_uninterpreted_fns";
// refresh all model regulations
const REFRESH_REGULATIONS_PATH: &str = "get_regulations";
// refresh all model layouts
const REFRESH_LAYOUTS_PATH: &str = "get_layouts";
// refresh all nodes in a particular layout
const REFRESH_LAYOUT_NODES_PATH: &str = "get_layout_nodes";

impl SessionHelper for ModelState {}

impl SessionState for ModelState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        match at_path.first() {
            Some(&VAR_EVENT_PATH) => self.perform_variable_event(event, &at_path[1..]),
            Some(&FN_EVENT_PATH) => self.perform_uninterpreted_fn_event(event, &at_path[1..]),
            Some(&REGULATION_EVENT_PATH) => self.perform_regulation_event(event, &at_path[1..]),
            Some(&LAYOUT_EVENT_PATH) => self.perform_layout_event(event, &at_path[1..]),
            _ => Self::invalid_path_error_generic(at_path),
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        match at_path.first() {
            Some(&REFRESH_MODEL_PATH) => self.refresh_whole_model(full_path),
            Some(&REFRESH_VARS_PATH) => self.refresh_variables(full_path),
            Some(&REFRESH_FNS_PATH) => self.refresh_uninterpreted_fns(full_path),
            Some(&REFRESH_REGULATIONS_PATH) => self.refresh_regulations(full_path),
            Some(&REFRESH_LAYOUTS_PATH) => self.refresh_layouts(full_path),
            Some(&REFRESH_LAYOUT_NODES_PATH) => self.refresh_layout_nodes(full_path, &at_path[1..]),
            _ => Self::invalid_path_error_generic(at_path),
        }
    }
}
