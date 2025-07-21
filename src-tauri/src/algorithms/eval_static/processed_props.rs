use crate::algorithms::eval_static::encode::*;
use crate::algorithms::fo_logic::fol_tree::FolTreeNode;
use crate::algorithms::fo_logic::parser::parse_and_minimize_fol_formula;
use crate::sketchbook::ids::UninterpretedFnId;
use crate::sketchbook::model::FnTree;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::Sketch;

use biodivine_lib_param_bn::BooleanNetwork;
use std::collections::HashMap;

/// Processed static property encoded in FOL.
///
/// For now, we use FOL checking as the only evaluation method, so we only need a single
/// struct for processed static properties. If we ever add more evaluation methods, we
/// can make this struct into an enum (like the one we use for dynamic props).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedStatProp {
    pub id: String,
    pub formula: FolTreeNode,
}

/// Simplified constructors for processed dynamic properties.
impl ProcessedStatProp {
    /// Create FOL `ProcessedStatProp` instance.
    pub fn mk_fol(id: &str, formula: FolTreeNode) -> ProcessedStatProp {
        ProcessedStatProp {
            id: id.to_string(),
            formula,
        }
    }

    /// Get ID of the processed property.
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Process static properties from a sketch, converting them into a more suitable
/// version for the evaluation. Currently, all properties are encoded into FOL, but we
/// might support some other kind of preprocessing in future.
///
/// Since some function symbols of the sketch were unused (in any update functions) and
/// filtered out when creating the BN, we must also get rid of any static properties referencing
/// these symbols. Note that removing properties of unused functions has no effect on the results.
///
/// TODO: Pruned symbols that were replaced by their expression must be substituted with that expression
/// TODO v2: What to do with properties referencing symbols that were pruned because they were unused?
pub fn process_static_props(
    sketch: &Sketch,
    bn: &BooleanNetwork,
    _fn_expressions: &HashMap<UninterpretedFnId, Option<FnTree>>,
    base_var_name: &str,
) -> Result<Vec<ProcessedStatProp>, String> {
    let mut static_props = sketch.properties.stat_props().collect::<Vec<_>>();
    // Sort properties by IDs for deterministic computation times (and get rid of the IDs)
    static_props.sort_by(|(a_id, _), (b_id, _)| a_id.cmp(b_id));

    let mut processed_props = Vec::new();
    for (id, stat_prop) in static_props {
        // TODO: Below is a temporary solution, the function symbols should be substituted with their
        //       expressions after being transformed into FOL.
        //       We want to remove all properties of function symbols (parameters) not present in the BN.
        //       These parameters were pruned beforehand because they are unused (redundant), and
        //       we must do the same with their properties.

        // Everything else is currently encoded into first-order logic (into a "generic" property)
        let formula_str = match stat_prop.get_prop_data() {
            StatPropertyType::GenericStatProp(prop) => {
                // TODO: What to do with pruned symbols present in formula?

                // Just take the formula from the generic property (we'll parse and minimize it below)
                prop.processed_formula.tree().to_string()
            }
            StatPropertyType::RegulationEssential(prop)
            | StatPropertyType::RegulationEssentialContext(prop) => {
                let input_name = prop.input.clone().unwrap();
                let target_name = prop.target.clone().unwrap();
                let mut formula = encode_regulation_essentiality(
                    input_name.as_str(),
                    target_name.as_str(),
                    prop.clone().value,
                    bn,
                );
                if let Some(context_formula) = &prop.context {
                    formula = encode_property_in_context(context_formula, &formula);
                }
                formula
            }
            StatPropertyType::RegulationMonotonic(prop)
            | StatPropertyType::RegulationMonotonicContext(prop) => {
                let input_name = prop.input.clone().unwrap();
                let target_name = prop.target.clone().unwrap();
                let mut formula = encode_regulation_monotonicity(
                    input_name.as_str(),
                    target_name.as_str(),
                    prop.clone().value,
                    bn,
                );
                if let Some(context_formula) = &prop.context {
                    formula = encode_property_in_context(context_formula, &formula);
                }
                formula
            }
            StatPropertyType::FnInputEssential(prop)
            | StatPropertyType::FnInputEssentialContext(prop) => {
                let fn_id = prop.target.clone().unwrap();
                // Only process this property if the BN contains this function symbol as its valid
                // parameter (otherwise it was pruned out for being unused).

                // TODO: Once we add substitutions for pruned symbols, update this
                if bn.find_parameter(fn_id.as_str()).is_some() {
                    let input_idx = prop.input_index.unwrap();
                    let number_inputs = sketch.model.get_uninterpreted_fn_arity(&fn_id)?;
                    let mut formula = encode_essentiality(
                        number_inputs,
                        input_idx,
                        fn_id.as_str(),
                        prop.clone().value,
                    );
                    if let Some(context_formula) = &prop.context {
                        formula = encode_property_in_context(context_formula, &formula);
                    }
                    formula
                } else {
                    // TODO: There will go the substitution somewhere
                    "true".to_string()
                }
            }
            StatPropertyType::FnInputMonotonic(prop)
            | StatPropertyType::FnInputMonotonicContext(prop) => {
                let fn_id = prop.target.clone().unwrap();
                // Only process this property if the BN contains this function symbol as its valid
                // parameter (otherwise it was pruned out for being unused).

                // TODO: Once we add substitutions for pruned symbols, update this
                if bn.find_parameter(fn_id.as_str()).is_some() {
                    let input_idx = prop.input_index.unwrap();
                    let number_inputs = sketch.model.get_uninterpreted_fn_arity(&fn_id)?;
                    let mut formula = encode_monotonicity(
                        number_inputs,
                        input_idx,
                        fn_id.as_str(),
                        prop.clone().value,
                    );
                    if let Some(context_formula) = &prop.context {
                        formula = encode_property_in_context(context_formula, &formula);
                    }
                    formula
                } else {
                    // TODO: There will go the substitution somewhere
                    "true".to_string()
                }
            }
        };
        let parsed_expression = parse_and_minimize_fol_formula(&formula_str, base_var_name)?;
        let new_prop = ProcessedStatProp::mk_fol(id.as_str(), parsed_expression);
        processed_props.push(new_prop);
    }

    Ok(processed_props)
}
