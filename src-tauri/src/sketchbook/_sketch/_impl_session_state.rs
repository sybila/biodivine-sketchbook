use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::event_utils::{make_reversible, make_state_change};
use crate::sketchbook::{JsonSerde, Sketch};
use base64::prelude::*;
use std::fs::File;
use std::io::Read;

/* Constants for event path segments for various events. */

// events being delegated to `model` subcomponent
const MODEL_PATH: &str = "model";
// events being delegated to `observations` subcomponent
const OBSERVATIONS_PATH: &str = "observations";
// events being delegated to `properties` subcomponent
const PROPERTIES_PATH: &str = "properties";
// create new sketch and replace the current data
const NEW_SKETCH_PATH: &str = "new_sketch";
// export the current sketch to custom format
const EXPORT_SKETCH_PATH: &str = "export_sketch";
// export the current sketch to extended aeon format
const EXPORT_AEON_PATH: &str = "export_aeon";
// export the provided network data into PNG
const EXPORT_PNG_PATH: &str = "export_png";
// import sketch from custom format and replace the current data
const IMPORT_SKETCH_PATH: &str = "import_sketch";
// import sketch from aeon format and replace the current data
const IMPORT_AEON_PATH: &str = "import_aeon";
// import model from sbml format and replace the current data
const IMPORT_SBML_PATH: &str = "import_sbml";
// check if various components of sketch are consistent together (and report issues)
const CHECK_CONSISTENCY_PATH: &str = "check_consistency";
// get number of parameters of the PSBN
const GET_NUM_PSBN_PARAMS_PATH: &str = "get_num_psbn_params";
// assert that various components of sketch are consistent together
const ASSERT_CONSISTENCY_PATH: &str = "assert_consistency";
// set annotation for the sketch
const SET_ANNOTATION_PATH: &str = "set_annotation";
// refresh the whole sketch
const GET_WHOLE_SKETCH_PATH: &str = "get_whole_sketch";

impl SessionHelper for Sketch {}

impl SessionState for Sketch {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        // just distribute the events one layer down, or answer some specific cases
        if let Some(at_path) = Self::starts_with(MODEL_PATH, at_path) {
            self.model.perform_event(event, at_path)
        } else if let Some(at_path) = Self::starts_with(OBSERVATIONS_PATH, at_path) {
            self.observations.perform_event(event, at_path)
        } else if let Some(at_path) = Self::starts_with(PROPERTIES_PATH, at_path) {
            self.properties.perform_event(event, at_path)
        } else if Self::starts_with(NEW_SKETCH_PATH, at_path).is_some() {
            self.set_to_empty();
            let sketch_data = SketchData::new_from_sketch(self);
            let state_change = make_state_change(&["sketch", "set_all"], &sketch_data);
            // this is probably one of the real irreversible changes
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with(EXPORT_SKETCH_PATH, at_path).is_some() {
            let path = Self::clone_payload_str(event, "sketch")?;
            self.export_to_custom_json(&path)?;
            Ok(Consumed::NoChange)
        } else if Self::starts_with(EXPORT_AEON_PATH, at_path).is_some() {
            let path = Self::clone_payload_str(event, "sketch")?;
            self.export_to_aeon(&path)?;
            Ok(Consumed::NoChange)
        } else if Self::starts_with(EXPORT_PNG_PATH, at_path).is_some() {
            // get payload and parse the path and png data (base64 encoded)
            let payload = Self::clone_payload_str(event, "sketch")?;
            let payload_json: serde_json::Value = serde_json::from_str(&payload)?;
            let path = payload_json["path"]
                .as_str()
                .ok_or("Missing 'path' in payload")?;
            let png_base64 = payload_json["png"]
                .as_str()
                .ok_or("Missing 'png' in payload")?;
            // decode the base64 data and write it to the file
            let png_data = BASE64_STANDARD.decode(png_base64)?;
            std::fs::write(path, png_data)?;
            Ok(Consumed::NoChange)
        } else if Self::starts_with(IMPORT_SKETCH_PATH, at_path).is_some() {
            let file_path = Self::clone_payload_str(event, "sketch")?;
            // read the file contents
            let mut file = File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // parse the SketchData, modify the sketch
            let new_sketch = Sketch::from_custom_json(&contents)?;
            self.modify_from_sketch(&new_sketch);

            let sketch_data = SketchData::new_from_sketch(self);
            let state_change = make_state_change(&["sketch", "set_all"], &sketch_data);
            // this is probably one of the real irreversible changes
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with(IMPORT_AEON_PATH, at_path).is_some() {
            let file_path = Self::clone_payload_str(event, "sketch")?;
            // read the file contents
            let mut file = File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // parse AEON format (extended with custom annotations)
            let new_sketch = Sketch::from_aeon(&contents)?;
            self.modify_from_sketch(&new_sketch);

            let sketch_data = SketchData::new_from_sketch(self);
            let state_change = make_state_change(&["sketch", "set_all"], &sketch_data);
            // this is probably one of the real irreversible changes
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with(IMPORT_SBML_PATH, at_path).is_some() {
            let file_path = Self::clone_payload_str(event, "sketch")?;
            // read the file contents
            let mut file = File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // parse the SBML format (only psbn, no additional properties or datasets)
            let new_sketch = Sketch::from_sbml(&contents)?;
            self.modify_from_sketch(&new_sketch);

            let sketch_data = SketchData::new_from_sketch(self);
            let state_change = make_state_change(&["sketch", "set_all"], &sketch_data);
            // this is probably one of the real irreversible changes
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with(CHECK_CONSISTENCY_PATH, at_path).is_some() {
            let (success, main_message, warn_message) = self.run_consistency_check();
            let results = if success {
                "No major issues with the sketch were discovered!".to_string()
            } else {
                format!("There are major issues with the sketch:\n\n{main_message}")
            };

            let payload = serde_json::to_string(&results).unwrap();
            let state_change = Event::build(&["sketch", "consistency_results"], Some(&payload));

            // Result is an irreversible event that just bypasses the stack (but does not reset it)
            // We sent the state change with or without warning message, depending if its empty
            if warn_message.is_empty() {
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: false,
                })
            } else {
                let warning =
                    format!("The sketch has potential minor issues. Please review before running inference: \n\n{warn_message}");
                Ok(Consumed::IrreversibleWithWarning {
                    state_change,
                    reset: false,
                    warning,
                })
            }
        } else if Self::starts_with(GET_NUM_PSBN_PARAMS_PATH, at_path).is_some() {
            let num_params = self.get_num_parameters();
            let payload = serde_json::to_string(&num_params).unwrap();
            let state_change = Event::build(&["sketch", "num_psbn_params"], Some(&payload));
            // irreversible event that just bypasses the stack (but does not reset it)
            Ok(Consumed::Irreversible {
                state_change,
                reset: false,
            })
        } else if Self::starts_with(SET_ANNOTATION_PATH, at_path).is_some() {
            let new_annotation = Self::clone_payload_str(event, "sketch")?;
            let orig_annotation = self.get_annotation().to_string();
            if new_annotation == orig_annotation {
                return Ok(Consumed::NoChange);
            }

            // set the annotation and prepare state-change + reverse events
            self.set_annotation(&new_annotation);
            let payload = serde_json::to_string(&new_annotation).unwrap();
            let state_change = Event::build(&["sketch", "set_annotation"], Some(&payload));
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(orig_annotation);

            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(ASSERT_CONSISTENCY_PATH, at_path).is_some() {
            // This is a "synthetic" event that either returns an error, or Consumed::NoChange
            // Use `CHECK_CONSISTENCY_PATH` to also send detailed message to frontend
            self.assert_consistency()?;
            Ok(Consumed::NoChange)
        } else {
            Self::invalid_path_error_generic(at_path)
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        // just distribute the events one layer down, or answer some specific cases
        if let Some(at_path) = Self::starts_with(MODEL_PATH, at_path) {
            self.model.refresh(full_path, at_path)
        } else if let Some(at_path) = Self::starts_with(OBSERVATIONS_PATH, at_path) {
            self.observations.refresh(full_path, at_path)
        } else if let Some(at_path) = Self::starts_with(PROPERTIES_PATH, at_path) {
            self.properties.refresh(full_path, at_path)
        } else if Self::starts_with(GET_WHOLE_SKETCH_PATH, at_path).is_some() {
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
