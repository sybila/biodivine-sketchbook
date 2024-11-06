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
    ///
    /// This format includes the standard AEON format for PSBN and layout.
    /// This format is compatible with other biodivine tools, but might not cover all
    /// parts of the sketch.
    /// 
    /// Apart from that, all details of the sketch are given via model annotations.
    /// Currently the annotations are given simpy as `component_type: id: json_string`.
    /// These components can be variables, functions, properties, of datasets.
    pub fn to_aeon(&self) -> String {
        // for standard part of aeon format, we use the transformation into aeon BN
        // this loses some info (like new regulation types), but that is preserved via annotations
        let bn = self.model.to_bn();
        let mut aeon_str = bn.to_string();

        // set layout info
        let default_layout = self.model.get_default_layout();
        for (var_id, node) in default_layout.layout_nodes() {
            // write position in format #position:ID:X,Y
            let pos = node.get_position();
            let node_layout_str = format!("#position:{var_id}:{},{}\n", pos.0, pos.1);
            aeon_str.push_str(&node_layout_str);
        }

        // set the rest using aeon model annotations
        let mut annotation = ModelAnnotation::new();

        // set static properties
        for (id, stat_prop) in self.properties.stat_props() {
            let prop_data_json = StatPropertyData::from_property(id, stat_prop).to_json_str();
            annotation.ensure_value(&["static_property", id.as_str()], &prop_data_json);
        }
        // set dynamic properties
        for (id, dyn_prop) in self.properties.dyn_props() {
            let prop_data_json = DynPropertyData::from_property(id, dyn_prop).to_json_str();
            annotation.ensure_value(&["dynamic_property", id.as_str()], &prop_data_json);
        }
        // set datasets
        for (id, dataset) in self.observations.datasets() {
            let dataset_data_json = DatasetData::from_dataset(id, dataset).to_json_str();
            annotation.ensure_value(&["dataset", id.as_str()], &dataset_data_json);
        }
        // set variable details
        for (var_id, variable) in self.model.variables() {
            let update_fn = self.model.get_update_fn(var_id).unwrap();
            let var_data_json = VariableData::from_var(var_id, variable, update_fn).to_json_str();
            annotation.ensure_value(&["variable", var_id.as_str()], &var_data_json);
        }
        // set function details
        for (fn_id, uninterpreted_fn) in self.model.uninterpreted_fns() {
            let fn_data_json = UninterpretedFnData::from_fn(fn_id, uninterpreted_fn).to_json_str();
            annotation.ensure_value(&["function", fn_id.as_str()], &fn_data_json);
        }

        // push the annotations to the aeon string
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
