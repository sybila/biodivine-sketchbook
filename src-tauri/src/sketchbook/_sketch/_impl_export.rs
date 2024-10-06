use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::{JsonSerde, Sketch};
use std::fs::File;
use std::io::Write;

impl Sketch {
    /// Export the sketch instance into a custom JSON model format.
    ///
    /// See [SketchData::to_pretty_json_str] for details on the actual conversion to JSON.
    pub fn export_to_custom_json(&self, filepath: &str) -> Result<(), String> {
        let sketch_data = SketchData::new_from_sketch(self);
        let mut file = File::create(filepath).map_err(|e| e.to_string())?;
        // write sketch in JSON to the file
        file.write_all(sketch_data.to_pretty_json_str().as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
