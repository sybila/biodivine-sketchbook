use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::ids::{DynPropertyId, StatPropertyId};
use crate::sketchbook::model::{Essentiality, ModelState, Monotonicity};
use crate::sketchbook::stat_prop_utils::*;
use crate::sketchbook::{JsonSerde, Sketch};
use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};
use regex::Regex;

type NamedProperties = Vec<(String, String)>;

impl Sketch {
    /// Create sketch instance from a AEON model format.
    ///
    // TODO: aeon format currently does not support template properties and datasets.
    pub fn from_aeon(aeon_str: &str) -> Result<Sketch, String> {
        let mut sketch = Sketch::default();

        let bn = BooleanNetwork::try_from(aeon_str)?;
        let model = ModelState::from_bn(&bn)?;

        sketch.model = model;
        // correctly set regulation static properties if needed
        for reg in sketch.model.regulations() {
            let input_var = reg.get_regulator();
            let target_var = reg.get_target();

            if reg.get_essentiality() != &Essentiality::Unknown {
                let prop_id = get_essentiality_prop_id(input_var, target_var);
                let prop = mk_essentiality_prop(input_var, target_var, *reg.get_essentiality());
                sketch.properties.add_raw_static(prop_id, prop)?;
            }

            if reg.get_sign() != &Monotonicity::Unknown {
                let prop_id = get_monotonicity_prop_id(input_var, target_var);
                let prop = mk_monotonicity_prop(input_var, target_var, *reg.get_sign());
                sketch.properties.add_raw_static(prop_id, prop)?;
            }
        }

        // set layout info
        let node_positions = Self::extract_aeon_layout_info(aeon_str);
        let default_layout = ModelState::get_default_layout_id();
        for (node, x, y) in node_positions {
            let node_id = sketch.model.get_var_id(&node)?;
            sketch
                .model
                .update_position(&default_layout, &node_id, x, y)?;
        }

        // set generic static& dynamic properties from aeon model annotations
        let aeon_annotations = ModelAnnotation::from_model_string(aeon_str);
        let (stat_props, dyn_props) = Self::extract_model_properties(&aeon_annotations)?;
        for (name, formula) in stat_props {
            let id = StatPropertyId::new(&name)?;
            sketch.properties.add_stat_generic(id, &name, &formula)?
        }
        for (name, formula) in dyn_props {
            let id = DynPropertyId::new(&name)?;
            sketch.properties.add_dyn_generic(id, &name, &formula)?
        }

        Ok(sketch)
    }

    /// Extract positions of nodes from the aeon model string.
    /// Positions are lines `#position:NODE_ID:X,Y`.
    /// Return list of triplets <node_id, x, y>.
    fn extract_aeon_layout_info(aeon_str: &str) -> Vec<(String, f32, f32)> {
        let re = Regex::new(r"^#position:(\w+):([+-]?\d*\.\d+),([+-]?\d*\.\d+)$").unwrap();

        let mut positions = Vec::new();
        for line in aeon_str.lines() {
            let line = line.trim();
            if let Some(captures) = re.captures(line) {
                // Extract the NODE_ID, X, and Y values from the captures
                let node_id = captures.get(1).unwrap().as_str().to_string();
                let x = captures.get(2).unwrap().as_str().parse::<f32>().unwrap();
                let y = captures.get(3).unwrap().as_str().parse::<f32>().unwrap();
                positions.push((node_id, x, y))
            }
        }
        positions
    }

    /// Extract two lists of named properties (static and dynamic) from an `.aeon` model
    /// annotation object.
    ///
    /// The properties are expected to appear as one of:
    /// - `#!dynamic_property: NAME: HCTL_FORMULA` for dynamic properties.
    /// - `#!static_property: NAME: FOL_FORMULA` for static properties.
    ///
    /// Each list is returned in alphabetic order w.r.t. the property name.
    fn extract_model_properties(
        annotations: &ModelAnnotation,
    ) -> Result<(NamedProperties, NamedProperties), String> {
        let stat_props = if let Some(property_node) = annotations.get_child(&["static_property"]) {
            Self::process_property_node(property_node)?
        } else {
            Vec::new()
        };
        let dyn_props = if let Some(property_node) = annotations.get_child(&["dynamic_property"]) {
            Self::process_property_node(property_node)?
        } else {
            Vec::new()
        };
        Ok((stat_props, dyn_props))
    }

    /// Given a `ModelAnnotation` node corresponding to `dynamic_property` or `static_property`,
    /// collect all named properties from its child nodes.
    ///
    /// This is possible because both `dynamic_property` and `static_property` annotations
    /// share common structure.
    ///
    /// The properties are expected to appear as one of:
    /// - `#!dynamic_property: NAME: FORMULA` for dynamic properties.
    /// - `#!static_property: NAME: FORMULA` for static properties.
    ///
    /// Each list is returned in alphabetic order w.r.t. the property name.
    fn process_property_node(
        property_node: &ModelAnnotation,
    ) -> Result<Vec<(String, String)>, String> {
        let mut properties = Vec::with_capacity(property_node.children().len());
        for (name, child) in property_node.children() {
            if !child.children().is_empty() {
                return Err(format!("Property `{name}` contains nested values."));
            }
            let Some(value) = child.value() else {
                return Err(format!("Found empty dynamic property `{name}`."));
            };
            if value.lines().count() > 1 {
                return Err(format!("Found multiple properties named `{name}`."));
            }
            properties.push((name.clone(), value.clone()));
        }
        // Sort alphabetically to avoid possible non-determinism down the line.
        properties.sort_by(|(x, _), (y, _)| x.cmp(y));
        Ok(properties)
    }
}

impl Sketch {
    /// Create sketch instance from a custom JSON model format.
    ///
    /// See [SketchData::from_json_str] for details on the actual parsing.
    pub fn from_custom_json(json_str: &str) -> Result<Sketch, String> {
        // parse the JSON to intermediate SketchData first
        let sketch_data = SketchData::from_json_str(json_str)?;
        let sketch = Sketch::new_from_sketch_data(&sketch_data)?;
        Ok(sketch)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::Sketch;
    use std::fs::File;
    use std::io::Read;

    #[test]
    /// Test that importing the same data from two different formats results in the same sketch.
    fn sketch_import() {
        let mut aeon_sketch_file = File::open("../data/test_data/test_model.aeon").unwrap();
        let mut json_sketch_file = File::open("../data/test_data/test_model.json").unwrap();

        let mut aeon_contents = String::new();
        aeon_sketch_file.read_to_string(&mut aeon_contents).unwrap();
        let mut json_contents = String::new();
        json_sketch_file.read_to_string(&mut json_contents).unwrap();

        let sketch1 = Sketch::from_aeon(&aeon_contents).unwrap();
        let sketch2 = Sketch::from_custom_json(&json_contents).unwrap();

        assert_eq!(sketch1, sketch2);
    }
}
