use biodivine_lib_param_bn::ModelAnnotation;

use crate::sketchbook::data_structs::{
    DatasetData, DynPropertyData, SketchData, StatPropertyData, UninterpretedFnData, VariableData,
};
use crate::sketchbook::{JsonSerde, Sketch};
use std::fs::File;
use std::io::Write;

impl Sketch {
    /// Convert the sketch instance into a custom (pretty) JSON string.
    ///
    /// See [SketchData::to_pretty_json_str] for details on the actual conversion to JSON.
    pub fn to_custom_json(&self) -> String {
        let sketch_data = SketchData::new_from_sketch(self);
        sketch_data.to_pretty_json_str()
    }

    /// Export the sketch instance into a custom JSON model format.
    ///
    /// See [SketchData::to_pretty_json_str] for details on the actual conversion to JSON.
    pub fn export_to_custom_json(&self, filepath: &str) -> Result<(), String> {
        let json_str = self.to_custom_json();
        let mut file = File::create(filepath).map_err(|e| e.to_string())?;
        // write sketch in JSON to the file
        file.write_all(json_str.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Sketch {
    /// Convert the sketch instance into a customized version of AEON model format.
    /// This format extends the standard AEON format for PSBN and layout, with additional
    /// information present in model annotations.
    ///
    /// This format is compatible with other biodivine tools, but the core does not
    /// cover all parts of the sketch. The remaining parts of the sketch are given
    /// via model annotations. Currently the annotations are given in json simply as
    ///   #!entity_type: ID: #`json_string`#
    /// These entities can be variables, functions, static/dynamic properties, and datasets.
    ///
    /// Note that Sketchbook supports more options for regulation monotonicity/essentiality
    /// than the standard AEON format allows. These specialized (e.g., dual) regulations are
    /// made unspecified in the AEON core, but provided in full as part of the annotation.
    pub fn to_aeon(&self) -> String {
        // For standard part of aeon format, we use the transformation into aeon BN
        // This loses some info (specialized regulation types), but that is preserved via annotations
        let bn = self.model.to_bn();
        let mut aeon_str = bn.to_string();

        // Set layout info
        let default_layout = self.model.get_default_layout();
        for (var_id, node) in default_layout.layout_nodes() {
            // write position in format #position:ID:X,Y
            let pos = node.get_position();
            let node_layout_str = format!("#position:{var_id}:{},{}\n", pos.0, pos.1);
            aeon_str.push_str(&node_layout_str);
        }

        // Set the rest using aeon model annotations
        let mut annotation = ModelAnnotation::new();

        // Annotations for static properties
        for (id, stat_prop) in self.properties.stat_props() {
            let prop_data_json = StatPropertyData::from_property(id, stat_prop).to_json_str();
            annotation.ensure_value(&["static_property", id.as_str()], &prop_data_json);
        }
        // Annotations for dynamic properties
        for (id, dyn_prop) in self.properties.dyn_props() {
            let prop_data_json = DynPropertyData::from_property(id, dyn_prop).to_json_str();
            annotation.ensure_value(&["dynamic_property", id.as_str()], &prop_data_json);
        }
        // Annotations for datasets
        for (id, dataset) in self.observations.datasets() {
            let dataset_data_json = DatasetData::from_dataset(id, dataset).to_json_str();
            annotation.ensure_value(&["dataset", id.as_str()], &dataset_data_json);
        }
        // Annotations for variable details
        for (var_id, variable) in self.model.variables() {
            let update_fn = self.model.get_update_fn(var_id).unwrap();
            let var_data_json = VariableData::from_var(var_id, variable, update_fn).to_json_str();
            annotation.ensure_value(&["variable", var_id.as_str()], &var_data_json);
        }
        // Annotations for function details
        for (fn_id, uninterpreted_fn) in self.model.uninterpreted_fns() {
            let fn_data_json = UninterpretedFnData::from_fn(fn_id, uninterpreted_fn).to_json_str();
            annotation.ensure_value(&["function", fn_id.as_str()], &fn_data_json);
        }

        // Push the annotations to the aeon string
        let annotation_str = annotation.to_string();
        aeon_str.push_str(&annotation_str);
        aeon_str
    }

    /// Export the sketch instance into a customized version of AEON model format.
    ///
    /// See [Sketch::to_aeon] for details on the actual conversion.
    pub fn export_to_aeon(&self, filepath: &str) -> Result<(), String> {
        let aeon_str = self.to_aeon();
        let mut file = File::create(filepath).map_err(|e| e.to_string())?;
        // write sketch in AEON to the file
        file.write_all(aeon_str.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
