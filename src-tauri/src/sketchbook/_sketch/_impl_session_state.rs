use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::event_utils::make_state_change;
use crate::sketchbook::{JsonSerde, Sketch};
use std::fs::File;
use std::io::Read;

impl SessionHelper for Sketch {}

impl SessionState for Sketch {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        // just distribute the events one layer down, or answer some specific cases
        if let Some(at_path) = Self::starts_with("model", at_path) {
            self.model.perform_event(event, at_path)
        } else if let Some(at_path) = Self::starts_with("observations", at_path) {
            self.observations.perform_event(event, at_path)
        } else if let Some(at_path) = Self::starts_with("properties", at_path) {
            self.properties.perform_event(event, at_path)
        } else if Self::starts_with("new_sketch", at_path).is_some() {
            self.set_to_empty();
            let sketch_data = SketchData::new_from_sketch(self);
            let state_change = make_state_change(&["sketch", "set_all"], &sketch_data);
            // this is probably one of the real irreversible changes
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with("export_sketch", at_path).is_some() {
            let path = Self::clone_payload_str(event, "sketch")?;
            self.export_to_custom_json(&path)?;
            Ok(Consumed::NoChange)
        } else if Self::starts_with("import_sketch", at_path).is_some() {
            let file_path = Self::clone_payload_str(event, "sketch")?;
            // read the file contents
            let mut file = File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // parse the SketchData, modify the sketch
            let sketch_data = SketchData::from_json_str(&contents)?;
            self.modify_from_sketch_data(&sketch_data)?;

            let state_change = make_state_change(&["sketch", "set_all"], &sketch_data);
            // this is probably one of the real irreversible changes
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with("import_aeon", at_path).is_some() {
            let file_path = Self::clone_payload_str(event, "sketch")?;
            // read the file contents
            let mut file = File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // parse the AEON format
            // TODO: aeon format currently does not support template properties and datasets
            let new_sketch = Sketch::from_aeon(&contents)?;
            self.modify_from_sketch(&new_sketch);

            let sketch_data = SketchData::new_from_sketch(self);
            let state_change = make_state_change(&["sketch", "set_all"], &sketch_data);
            // this is probably one of the real irreversible changes
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with("check_consistency", at_path).is_some() {
            let (success, message) = self.run_consistency_check();
            let results = if success {
                "No issues with the sketch were discovered!".to_string()
            } else {
                format!("There are issues with the sketch:\n\n{message}")
            };

            let payload = serde_json::to_string(&results).unwrap();
            let state_change = Event::build(&["sketch", "consistency_results"], Some(&payload));
            // irreversible change that should just bypass the stack (not reset it)
            Ok(Consumed::Irreversible {
                state_change,
                reset: false,
            })
        } else if Self::starts_with("assert_consistency", at_path).is_some() {
            // this is a "synthetic" event that either returns an error, or Consumed::NoChange
            self.assert_consistency()?;
            Ok(Consumed::NoChange)
        } else {
            Self::invalid_path_error_generic(at_path)
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        // just distribute the events one layer down, or answer some specific cases
        if let Some(at_path) = Self::starts_with("model", at_path) {
            self.model.refresh(full_path, at_path)
        } else if let Some(at_path) = Self::starts_with("observations", at_path) {
            self.observations.refresh(full_path, at_path)
        } else if let Some(at_path) = Self::starts_with("properties", at_path) {
            self.properties.refresh(full_path, at_path)
        } else if Self::starts_with("get_whole_sketch", at_path).is_some() {
            let sketch_data = SketchData::new_from_sketch(self);
            Ok(Event {
                path: full_path.to_vec(),
                payload: Some(sketch_data.to_json_str()),
            })
        } else {
            Self::invalid_path_error_generic(at_path)
        }
    }
}
