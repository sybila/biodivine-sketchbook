use crate::sketchbook::data_structs::{
    DatasetData, DynPropertyData, SketchData, StatPropertyData, StatPropertyTypeData,
    UninterpretedFnData, VariableData,
};
use crate::sketchbook::ids::StatPropertyId;
use crate::sketchbook::model::{Essentiality, ModelState, Monotonicity};
use crate::sketchbook::observations::ObservationManager;
use crate::sketchbook::properties::shortcuts::*;
use crate::sketchbook::properties::{DynProperty, StatProperty};
use crate::sketchbook::{JsonSerde, Sketch};
use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};
use regex::Regex;
use std::fs::File;
use std::io::Read;

impl Sketch {
    /// Create sketch instance from a custom JSON model format.
    ///
    /// See [SketchData::from_json_str] for details on the actual parsing.
    pub fn from_custom_json(json_str: &str) -> Result<Sketch, String> {
        // parse the JSON to intermediate SketchData first
        let sketch_data = SketchData::from_json_str(json_str)?;
        let mut sketch = Sketch::new_from_sketch_data(&sketch_data)?;
        sketch.standardize_generated_static_ids()?;
        Ok(sketch)
    }

    /// Create sketch instance from a customized version of AEON model format.
    /// The original part of the AEON format (compatible with other biodivine tools) includes:
    /// - variable IDs
    /// - regulations (and corresponding automatically generated static properties)
    /// - update functions and function symbols
    /// - layout information
    ///
    /// The custom extension covers most remaining parts of the sketch via model annotations.
    /// Currently the annotations are given simpy as
    ///   #!entity_type: ID: #`json_string`#
    /// These annotations either cover additional information (complementing variables and
    /// functions), or completely new components like static/dynamic properties and datasets.
    ///
    /// If there are inconsistencies between annotations and pure AEON format, the annotations
    /// take precedence. This may arise since Sketchbook provides more types of regulations
    /// (e.g., dual) than AEON format can handle, and these are saved in the annotations.
    ///
    /// We allow a special case for writing HCTL and FOL properties, compatible with the
    /// original BN sketches prototype format:
    ///   #!static_property: ID: #`fol_formula_string`#
    ///   #!dynamic_property: ID: #`hctl_formula_string`#
    pub fn from_aeon(aeon_str: &str) -> Result<Sketch, String> {
        // Parse basic AEON PSBN info (variables, functions, regulations) from the aeon file.
        // This also derives automatically-generated regulation properties.
        // BUT we have to be careful - some function symbols of the sketch file may only
        // be present in the annotations (if they are not part of any update expressions).
        let bn = BooleanNetwork::try_from(aeon_str)?;
        let mut sketch = Sketch::from_boolean_network(&bn)?;

        // Set layout info
        let node_positions = Self::extract_aeon_layout_info(aeon_str);
        let default_layout = ModelState::get_default_layout_id();
        for (node, x, y) in node_positions {
            let node_id = sketch.model.get_var_id(&node)?;
            sketch
                .model
                .update_position(&default_layout, &node_id, x, y)?;
        }

        // Recover the remaining sketch components from aeon model annotations
        let aeon_annotations = ModelAnnotation::from_model_string(aeon_str);
        let variables = Self::extract_entities(&aeon_annotations, "variable")?;
        let functions = Self::extract_entities(&aeon_annotations, "function")?;
        let datasets = Self::extract_entities(&aeon_annotations, "dataset")?;
        let stat_props = Self::extract_entities(&aeon_annotations, "static_property")?;
        let dyn_props = Self::extract_entities(&aeon_annotations, "dynamic_property")?;

        // The annotations may contain some additional function symbols of the sketch (that
        // were not part of any update expressions). We must first add these function symbols
        // before we set any expressions that may involve them.
        for (id, fn_str) in functions.iter() {
            if !sketch.model.is_valid_uninterpreted_fn_id_str(id) {
                let fn_data = UninterpretedFnData::from_json_str(fn_str)?;
                let arity = fn_data.arguments.len();
                sketch
                    .model
                    .add_empty_uninterpreted_fn_by_str(id, id, arity)?;
            }
        }

        // Now we can safely add all additional info (like names, annotations, expressions)
        // for variables and uninterpreted functions.
        for (id, fn_str) in functions {
            let fn_data = UninterpretedFnData::from_json_str(&fn_str)?;
            let fn_id = sketch.model.get_uninterpreted_fn_id(&id)?;
            sketch
                .model
                .set_raw_function(&fn_id, fn_data.to_uninterpreted_fn(&sketch.model)?)?;
        }
        for (id, variable_str) in variables {
            let var_data = VariableData::from_json_str(&variable_str)?;
            let var_id = sketch.model.get_var_id(&id)?;
            sketch.model.set_raw_var(&var_id, var_data.to_var()?)?;
            sketch.model.set_update_fn(&var_id, &var_data.update_fn)?;
        }

        // Datasets have to be added from scratch
        for (id, dataset_str) in datasets {
            let dataset_data = DatasetData::from_json_str(&dataset_str)?;
            sketch
                .observations
                .add_dataset_by_str(&id, dataset_data.to_dataset()?)?;
        }

        // Properties have to be added from scratch (apart from automatically generated static props).
        // We allow two modes - a JSON string for any property, or formula string for HCTL/FOL properties.
        for (id, content_str) in stat_props {
            // Try parsing a FOL formula, and if not successful, parse a JSON string
            if let Ok(prop) = StatProperty::try_mk_generic(&id, &content_str) {
                sketch.properties.add_static_by_str(&id, prop)?
            } else {
                let prop_data = StatPropertyData::from_json_str(&content_str)?;
                let property = prop_data.to_property()?;

                // Ignore automatically generated static props as they were already added before
                // together with regulations.
                // However, if some regulation property parsed from annotation is not consistent with the
                // underlying regulation, update the regulation to match it. Since pure AEON format does not
                // support full range of monotonicity/essentiality options, advanced properties are only
                // conserved correctly in annotations.
                match prop_data.variant {
                    // If the current regulation has different essentiality, update its value
                    StatPropertyTypeData::RegulationEssential(p) => {
                        let maybe_regulator = p.input;
                        let maybe_target = p.target;
                        // Only interested in fully specified regulation properties
                        if let (Some(regulator), Some(target)) = (maybe_regulator, maybe_target) {
                            let current_reg =
                                sketch.model.get_regulation_by_str(&regulator, &target)?;
                            if *current_reg.get_essentiality() != p.value {
                                // Change both the model regulation and the corresponding static property
                                sketch.model.change_regulation_essentiality_by_str(
                                    &regulator, &target, &p.value,
                                )?;
                                // The static property may or may not (if the essentiality was unknown) be present
                                let prop_id = StatPropertyId::new(&id)?;
                                if sketch.properties.is_valid_stat_property_id(&prop_id) {
                                    sketch.properties.set_stat_essentiality(&prop_id, p.value)?;
                                } else {
                                    sketch.properties.add_static_by_str(&id, property)?;
                                }
                            }
                        }
                    }
                    // If the current regulation has different monotonicity, update its value
                    StatPropertyTypeData::RegulationMonotonic(p) => {
                        let maybe_regulator = p.input;
                        let maybe_target = p.target;
                        // Only interested in fully specified regulation properties
                        if let (Some(regulator), Some(target)) = (maybe_regulator, maybe_target) {
                            let current_reg =
                                sketch.model.get_regulation_by_str(&regulator, &target)?;
                            if *current_reg.get_sign() != p.value {
                                // Change both the model regulation and the corresponding static property
                                sketch
                                    .model
                                    .change_regulation_sign_by_str(&regulator, &target, &p.value)?;
                                // The static property may or may not (if the monononicity was unknown) be present
                                let prop_id = StatPropertyId::new(&id)?;
                                if sketch.properties.is_valid_stat_property_id(&prop_id) {
                                    sketch.properties.set_stat_monotonicity(&prop_id, p.value)?;
                                } else {
                                    sketch.properties.add_static_by_str(&id, property)?;
                                }
                            }
                        }
                    }
                    // Add all other properties as-is
                    _ => {
                        sketch.properties.add_static_by_str(&id, property)?;
                    }
                }
            }
        }
        for (id, content_str) in dyn_props {
            // Try parsing a HCTL formula, and if not successful, parse a JSON string
            if let Ok(prop) = DynProperty::try_mk_generic(&id, &content_str) {
                sketch.properties.add_dynamic_by_str(&id, prop)?
            } else {
                let prop_data = DynPropertyData::from_json_str(&content_str)?;
                sketch
                    .properties
                    .add_dynamic_by_str(&id, prop_data.to_property()?)?;
            }
        }

        // lastly, make sure that automatically generated static properties have standardized IDs
        sketch.standardize_generated_static_ids()?;
        Ok(sketch)
    }

    /// Create sketch instance from a SBML model format. This variant includes:
    /// - variables
    /// - regulations (and corresponding automatically generated static properties)
    /// - update functions and function symbols
    /// - layout information
    pub fn from_sbml(sbml_str: &str) -> Result<Sketch, String> {
        // Set psbn info (variables, functions, regulations and corresponding properties)
        let (bn, layout_map) = BooleanNetwork::try_from_sbml(sbml_str)?;
        let mut sketch = Sketch::from_boolean_network(&bn)?;

        let default_layout = ModelState::get_default_layout_id();
        for (variable_node, (px, py)) in layout_map {
            let var_id = sketch.model.get_var_id(&variable_node)?;
            sketch
                .model
                .update_position(&default_layout, &var_id, px as f32, py as f32)?;
        }

        // Lastly, make sure that automatically generated static properties have standardized IDs
        // This is probably not needed for SBML at the moment, but we may need it later
        sketch.standardize_generated_static_ids()?;
        Ok(sketch)
    }

    /// Create `Sketch` instance from a BooleanNetwork instance of `lib-param-bn`.
    /// This includes processing:
    /// - variables
    /// - regulations (and corresponding automatically generated static properties)
    /// - update functions and function symbols
    pub fn from_boolean_network(bn: &BooleanNetwork) -> Result<Sketch, String> {
        let mut sketch = Sketch::default();
        let model = ModelState::from_bn(bn)?;

        sketch.model = model;
        // Correctly set regulation static properties if needed
        for reg in sketch.model.regulations() {
            let input_var = reg.get_regulator();
            let target_var = reg.get_target();

            if reg.get_essentiality() != &Essentiality::Unknown {
                let prop_id = StatProperty::get_reg_essentiality_prop_id(input_var, target_var);
                let prop = mk_reg_essentiality_prop(input_var, target_var, *reg.get_essentiality());
                sketch.properties.add_static(prop_id, prop)?;
            }

            if reg.get_sign() != &Monotonicity::Unknown {
                let prop_id = StatProperty::get_reg_monotonicity_prop_id(input_var, target_var);
                let prop = mk_reg_monotonicity_prop(input_var, target_var, *reg.get_sign());
                sketch.properties.add_static(prop_id, prop)?;
            }
        }
        Ok(sketch)
    }

    /// Extract positions of nodes from the aeon model string.
    /// Positions are expect as lines in forllowing format:
    ///   #position:NODE_ID:X,Y
    ///
    /// This funtction returns a list of triplets <node_id, x, y>.
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

    /// Extract list of named entities (tuples with id/content) from an `.aeon` model
    /// annotation object.
    ///
    /// The entities are expected to appear as:
    ///   #!entity_type: ID: #`CONTENT`#
    /// So, for example:
    ///   #!variable:ANT:#`{"id":"ANT","name":"ANT","annotation":"","update_fn":""}`#
    ///
    /// Each list is returned in alphabetic order w.r.t. the entity name.
    fn extract_entities(
        annotations: &ModelAnnotation,
        entity_type: &str,
    ) -> Result<Vec<(String, String)>, String> {
        if let Some(entity_node) = annotations.get_child(&[entity_type]) {
            Self::process_entity_node(entity_node, entity_type)
        } else {
            Ok(Vec::new())
        }
    }

    /// Given a `ModelAnnotation` node corresponding to a particular entity type (like 'variable'),
    /// collect all entities of given type from the child nodes.
    ///
    /// This is general for all entity types as annotations share common structure.
    ///   #!entity_type: ID: #`CONTENT`#
    ///
    /// List is returned in alphabetic order w.r.t. the property name.
    fn process_entity_node(
        enitity_node: &ModelAnnotation,
        enitity_type: &str,
    ) -> Result<Vec<(String, String)>, String> {
        let mut entities = Vec::with_capacity(enitity_node.children().len());
        for (id, child) in enitity_node.children() {
            if !child.children().is_empty() {
                return Err(format!("{enitity_type} `{id}` contains nested values."));
            }
            let Some(value) = child.value() else {
                return Err(format!("Found empty {enitity_type} `{id}`."));
            };
            if value.lines().count() > 1 {
                return Err(format!(
                    "Found multiple entities of type {enitity_type} with id `{id}`."
                ));
            }
            entities.push((id.clone(), value.clone()));
        }
        // Sort alphabetically to avoid possible non-determinism down the line.
        entities.sort_by(|(x, _), (y, _)| x.cmp(y));
        Ok(entities)
    }

    /// Standardize IDs of all types of automatically generated static properties.
    /// That is:
    /// - automatically generated regulation properties get ID `monotonicity_REGULATOR_TARGET`
    ///   or `essentiality_REGULATOR_TARGET`
    /// - automatically generated function properties get ID `fn_monotonicity_FUNCTION_INDEX`
    ///   or `fn_essentiality_FUNCTION_INDEX`
    ///
    /// This is important during import, as these IDs could have been "corrupted" by the user
    /// outside of Sketchbook. That could create some issues down the line.
    fn standardize_generated_static_ids(&mut self) -> Result<(), String> {
        self.properties.make_generated_reg_prop_ids_consistent().map_err(|e| format!("Some IDs of generated regulation properties are corrupted and we cant standardize them: {e}"))?;
        self.properties.make_generated_fn_prop_ids_consistent().map_err(|e| format!("Some IDs of generated function properties are corrupted and we cant standardize them: {e}"))?;
        Ok(())
    }

    /// Load dataset from a provided CSV file path, and add it (with provided id/name)
    /// directly to this sketch.
    pub fn load_dataset(&mut self, dataset_id: &str, csv_path: &str) -> Result<(), String> {
        // Load file contents
        let mut file = File::open(csv_path).map_err(|e| e.to_string())?;
        let mut csv_string = String::new();
        file.read_to_string(&mut csv_string)
            .map_err(|e| e.to_string())?;

        // Process the CSV data into `Dataset` instance and add it to the sketch
        let parsed_dataset =
            ObservationManager::parse_dataset_from_csv(dataset_id, &csv_string).unwrap();
        self.observations
            .add_dataset_by_str(dataset_id, parsed_dataset)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::Sketch;
    use std::fs::File;
    use std::io::Read;

    #[test]
    /// Test that importing the same data from different formats results in the same sketch.
    /// These models only include PSBN (no additional datasets or properties) with standard
    /// AEON-compatible regulations (no dual regulations and so on).
    fn basic_import() {
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

    #[test]
    /// Test that importing the same data from aeon and json format results in the same sketch.
    /// This test involves two full sketches, but only with standard AEON-compatible regulations
    /// (no dual regulations and so on).
    fn full_import() {
        for sketch_idx in [1, 2] {
            let mut aeon_sketch_file =
                File::open(format!("../data/test_data/test_sketch_{sketch_idx}.aeon")).unwrap();
            let mut json_sketch_file =
                File::open(format!("../data/test_data/test_sketch_{sketch_idx}.json")).unwrap();

            let mut aeon_contents = String::new();
            aeon_sketch_file.read_to_string(&mut aeon_contents).unwrap();
            let mut json_contents = String::new();
            json_sketch_file.read_to_string(&mut json_contents).unwrap();

            let sketch1 = Sketch::from_aeon(&aeon_contents).unwrap();
            let sketch2 = Sketch::from_custom_json(&json_contents).unwrap();
            assert_eq!(sketch1, sketch2);
        }
    }

    #[test]
    /// Test that importing the same data from aeon and json format results in the same sketch.
    /// This test involves two models with various non-aeon-compatible regulations (such as
    /// dual regulations).
    fn full_import_with_various_regulations() {
        let mut aeon_sketch_file =
            File::open("../data/test_data/test_model_various_regulations.aeon".to_string())
                .unwrap();
        let mut json_sketch_file =
            File::open("../data/test_data/test_model_various_regulations.json".to_string())
                .unwrap();

        let mut aeon_contents = String::new();
        aeon_sketch_file.read_to_string(&mut aeon_contents).unwrap();
        let mut json_contents = String::new();
        json_sketch_file.read_to_string(&mut json_contents).unwrap();

        let sketch1 = Sketch::from_aeon(&aeon_contents).unwrap();
        let sketch2 = Sketch::from_custom_json(&json_contents).unwrap();
        assert_eq!(sketch1, sketch2);
    }
}
