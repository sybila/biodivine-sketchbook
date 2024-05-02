use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::{DynPropertyData, StatPropertyData};
use crate::sketchbook::event_utils::make_refresh_event;
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

        // currently three options: get all datasets, a single dataset, a single observation
        match at_path.first() {
            Some(&"get_all_dynamic") => {
                Self::assert_path_length(at_path, 1, component_name)?;
                let mut properties_list: Vec<DynPropertyData> = self
                    .dyn_properties
                    .iter()
                    .map(|(id, prop)| DynPropertyData::from_property(id, prop))
                    .collect();
                // return the list sorted, so that it is deterministic
                properties_list.sort_by(|a, b| a.id.cmp(&b.id));
                make_refresh_event(full_path, properties_list)
            }
            Some(&"get_all_static") => {
                Self::assert_path_length(at_path, 1, component_name)?;
                let mut properties_list: Vec<StatPropertyData> = self
                    .stat_properties
                    .iter()
                    .map(|(id, prop)| StatPropertyData::from_property(id, prop))
                    .collect();
                // return the list sorted, so that it is deterministic
                properties_list.sort_by(|a, b| a.id.cmp(&b.id));
                make_refresh_event(full_path, properties_list)
            }
            _ => Self::invalid_path_error_generic(at_path),
        }
    }
}
