use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::ModelState;

/// **(internal)** Implementation for events related to `layouts` of the model.
pub mod _events_layout;
/// **(internal)** Implementation for events related to `regulations` of the model.
pub mod _events_regulations;
/// **(internal)** Implementation for events related to `uninterpreted functions` of the model.
pub mod _events_uninterpreted_fns;
/// **(internal)** Implementation for events related to `update functions` of the model.
pub mod _events_update_fns;
/// **(internal)** Implementation for events related to `variables` of the model.
pub mod _events_variables;
/// **(internal)** Implementation for `refresh` (getter) events.
pub mod _refresh_events;

/// **(internal)** Tests for the event-based API.
#[cfg(test)]
mod _tests;

impl SessionHelper for ModelState {}

impl SessionState for ModelState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        match at_path.first() {
            Some(&"variable") => self.perform_variable_event(event, &at_path[1..]),
            Some(&"uninterpreted_fn") => self.perform_uninterpreted_fn_event(event, &at_path[1..]),
            Some(&"regulation") => self.perform_regulation_event(event, &at_path[1..]),
            Some(&"layout") => self.perform_layout_event(event, &at_path[1..]),
            // todo: add events for update functions
            _ => Self::invalid_path_error_generic(at_path),
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        match at_path.first() {
            Some(&"get_variables") => self.refresh_variables(full_path),
            Some(&"get_uninterpreted_fns") => self.refresh_uninterpreted_fns(full_path),
            Some(&"get_regulations") => self.refresh_regulations(full_path),
            Some(&"get_layouts") => self.refresh_layouts(full_path),
            Some(&"get_layout_nodes") => self.refresh_layout_nodes(full_path, &at_path[1..]),
            // todo: add events for update functions
            _ => Self::invalid_path_error_generic(at_path),
        }
    }
}
