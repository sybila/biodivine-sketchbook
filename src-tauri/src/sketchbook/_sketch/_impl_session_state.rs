use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::event_utils::make_state_change;
use crate::sketchbook::{JsonSerde, Sketch};
use std::fs::File;
use std::io::{Read, Write};

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
            let sketch_data = SketchData::new(&self.model, &self.observations, &self.properties);
            let state_change = make_state_change(&["sketch", "set_all"], &sketch_data);
            // this is probably one of the real irreversible changes
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with("export_sketch", at_path).is_some() {
            let sketch_data = SketchData::new(&self.model, &self.observations, &self.properties);
            let path = Self::clone_payload_str(event, "sketch")?;
            let mut file = File::create(path).map_err(|e| e.to_string())?;
            // write sketch in JSON to the file
            file.write_all(sketch_data.to_pretty_json_str().as_bytes())
                .map_err(|e| e.to_string())?;
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
            let sketch_data = SketchData::new(&self.model, &self.observations, &self.properties);
            Ok(Event {
                path: full_path.to_vec(),
                payload: Some(sketch_data.to_json_str()),
            })
        } else {
            Self::invalid_path_error_generic(at_path)
        }
    }
}
