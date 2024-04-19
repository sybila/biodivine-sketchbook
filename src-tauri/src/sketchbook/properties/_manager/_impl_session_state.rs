use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::properties::PropertyManager;

impl SessionHelper for PropertyManager {}

impl SessionState for PropertyManager {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        let component_name = "properties";

        // TODO - make `perform` events available

        panic!(
            "{component_name} cannot consume {:?}, at {:?} no events for properties yet.",
            event, at_path
        );
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        let component_name = "properties";

        // TODO - make `refresh` events available

        panic!(
            "{component_name} cannot consume event with full path `{:?}` (at `{:?}`), there are no events for properties yet.",
            full_path,
            at_path
        );
    }
}
