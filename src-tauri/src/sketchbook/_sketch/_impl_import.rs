use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::model::{Essentiality, ModelState, Monotonicity};
use crate::sketchbook::properties::shortcuts::*;
use crate::sketchbook::properties::{DynProperty, StatProperty};
use crate::sketchbook::{JsonSerde, Sketch};
use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};
use regex::Regex;

type NamedProperties = Vec<(String, String)>;

impl Sketch {
    /// Create sketch instance from a AEON model format. This variant includes:
    /// - variables
    /// - regulations (and corresponding automatically generated static properties)
    /// - update functions and function symbols
    /// - layout information
    /// - HCTL dynamic properties
    /// - FOL static properties
    ///
    // TODO: our variant of aeon format currently does not consider template properties and datasets.
    // TODO: our variant of aeon format currently does not consider annotation.
    pub fn from_aeon(aeon_str: &str) -> Result<Sketch, String> {
        // set psbn info (variables, functions, regulations and corresponding properties)
        let bn = BooleanNetwork::try_from(aeon_str)?;
        let mut sketch = Sketch::from_boolean_network(&bn)?;

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
            let prop = StatProperty::try_mk_generic(&name, &formula, "")?;
            sketch.properties.add_static_by_str(&name, prop)?
        }
        for (name, formula) in dyn_props {
            let prop = DynProperty::try_mk_generic(&name, &formula, "")?;
            sketch.properties.add_dynamic_by_str(&name, prop)?
        }

        Ok(sketch)
    }

    /// Create sketch instance from a SBML model format. This variant includes:
    /// - variables
    /// - regulations (and corresponding automatically generated static properties)
    /// - update functions and function symbols
    /// - layout information
    pub fn from_sbml(sbml_str: &str) -> Result<Sketch, String> {
        // set psbn info (variables, functions, regulations and corresponding properties)
        let (bn, layout_map) = BooleanNetwork::try_from_sbml(sbml_str)?;
        let mut sketch = Sketch::from_boolean_network(&bn)?;

        let default_layout = ModelState::get_default_layout_id();
        for (node, (px, py)) in layout_map {
            let node_id = sketch.model.get_var_id(&node)?;
            sketch
                .model
                .update_position(&default_layout, &node_id, px as f32, py as f32)?;
        }

        Ok(sketch)
    }

    /// Create sketch instance from a BooleanNetwork instance of `lib-param-bn`.
    /// This includes processing:
    /// - variables
    /// - regulations (and corresponding automatically generated static properties)
    /// - update functions and function symbols
    pub fn from_boolean_network(bn: &BooleanNetwork) -> Result<Sketch, String> {
        let mut sketch = Sketch::default();
        let model = ModelState::from_bn(bn)?;

        sketch.model = model;
        // correctly set regulation static properties if needed
        for reg in sketch.model.regulations() {
            let input_var = reg.get_regulator();
            let target_var = reg.get_target();

            if reg.get_essentiality() != &Essentiality::Unknown {
                let prop_id = StatProperty::get_essentiality_prop_id(input_var, target_var);
                let prop = mk_essentiality_prop(input_var, target_var, *reg.get_essentiality());
                sketch.properties.add_static(prop_id, prop)?;
            }

            if reg.get_sign() != &Monotonicity::Unknown {
                let prop_id = StatProperty::get_monotonicity_prop_id(input_var, target_var);
                let prop = mk_monotonicity_prop(input_var, target_var, *reg.get_sign());
                sketch.properties.add_static(prop_id, prop)?;
            }
        }
        Ok(sketch)
    }

    /// Extract positions of nodes from the aeon model string.
    /// Positions are lines `#position:NODE_ID:X,Y`.
    /// Return list of triplets <node_id, x, y>.
    fn extract_aeon_layout_info(aeon_str: &str) -> Vec<(String, f32, f32)> {
        let re = Regex::new(r"^#position:(\w+):([+-]?\d+(\.\d+)?),([+-]?\d+(\.\d+)?)$").unwrap();

        let mut positions = Vec::new();
        for line in aeon_str.lines() {
            let line = line.trim();
            if let Some(captures) = re.captures(line) {
                // Extract the NODE_ID, X, and Y values from the captures
                let node_id = captures.get(1).unwrap().as_str().to_string();
                let x = captures
                    .get(2)
                    .unwrap()
                    .as_str()
                    .parse::<f32>()
                    .unwrap_or(0.0);
                let y = captures
                    .get(4)
                    .unwrap()
                    .as_str()
                    .parse::<f32>()
                    .unwrap_or(0.0);
                positions.push((node_id, x, y));
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
    /// Test that importing the same data from different formats results in the same sketch.
    /// These models only include PSBN (no additional datasets or porperties).
    fn sketch_import() {
        let mut aeon_sketch_file = File::open("../data/test_data/test_model.aeon").unwrap();
        let mut json_sketch_file = File::open("../data/test_data/test_model.json").unwrap();
        let mut sbml_sketch_file = File::open("../data/test_data/test_model.sbml").unwrap();

        let mut aeon_contents = String::new();
        aeon_sketch_file.read_to_string(&mut aeon_contents).unwrap();
        let mut json_contents = String::new();
        json_sketch_file.read_to_string(&mut json_contents).unwrap();
        let mut sbml_contents = String::new();
        sbml_sketch_file.read_to_string(&mut sbml_contents).unwrap();

        let sketch1 = Sketch::from_aeon(&aeon_contents).unwrap();
        let sketch2 = Sketch::from_custom_json(&json_contents).unwrap();
        let sketch3 = Sketch::from_sbml(&sbml_contents).unwrap();

        assert_eq!(sketch1, sketch2);
        assert_eq!(sketch2, sketch3);
    }
}
