use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::model::ModelState;
use crate::sketchbook::observations::ObservationManager;
use crate::sketchbook::properties::PropertyManager;
use crate::sketchbook::{JsonSerde, Manager};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

/// Object encompassing all of the individual modules of the Boolean network sketch.
///
/// Most of the actual functionality is implemented by the modules themselves, `Sketch`
/// currently only distributes events and handles situations when cooperation between
/// modules is needed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sketch {
    model: ModelState,
    observations: ObservationManager,
    properties: PropertyManager,
}

impl<'de> JsonSerde<'de> for Sketch {}
impl Manager for Sketch {}

impl Default for Sketch {
    /// Default empty sketch.
    fn default() -> Sketch {
        Sketch {
            model: ModelState::default(),
            observations: ObservationManager::default(),
            properties: PropertyManager::default(),
        }
    }
}

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
        } else if Self::starts_with("export_sketch", at_path).is_some() {
            let sketch_data = SketchData::new(&self.model, &self.observations, &self.properties);
            let path = Self::clone_payload_str(event, "sketch")?;
            let mut file = File::create(path).map_err(|e| e.to_string())?;
            // write sketch in JSON to the file
            file.write_all(sketch_data.to_json_str().as_bytes())
                .map_err(|e| e.to_string())?;
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
        } else if Self::starts_with("get_sketch", at_path).is_some() {
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
