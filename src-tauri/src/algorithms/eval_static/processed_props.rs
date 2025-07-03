use crate::algorithms::eval_static::encode::*;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::Sketch;

use biodivine_lib_param_bn::BooleanNetwork;

/// Processed static property encoded in FOL.
///
/// If we ever need more variants than just FOL prop, make this enum (like
/// the one used for dynamic props).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedStatProp {
    pub id: String,
    pub formula: String,
}

/// Simplified constructors for processed dynamic properties.
impl ProcessedStatProp {
    /// Create FOL `ProcessedStatProp` instance.
    pub fn mk_fol(id: &str, formula: &str) -> ProcessedStatProp {
        ProcessedStatProp {
            id: id.to_string(),
            formula: formula.to_string(),
        }
    }

    /// Get ID of the processed property.
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Process static properties from a sketch, converting them into one of the supported
/// `ProcessedStatProp` variants. Currently, all properties are encoded into FOL, but we
/// might support some other preprocessing in future.
///
/// Since some function symbols of the sketch were unused (in any update functions) and
/// filtered out when creating the BN, we must also get rid of any static properties referencing
/// these symbols. Note that removing properties of unused functions has no effect on the results.
///
/// TODO: Not sure how to handle generic FOL properties referencing pruned symbols.
/// TODO v2: Pruned symbols must be substituted with their expressions
pub fn process_static_props(
    sketch: &Sketch,
    bn: &BooleanNetwork,
) -> Result<Vec<ProcessedStatProp>, String> {
    let mut static_props = sketch.properties.stat_props().collect::<Vec<_>>();
    // sort properties by IDs for deterministic computation times (and get rid of the IDs)
    static_props.sort_by(|(a_id, _), (b_id, _)| a_id.cmp(b_id));

    let mut processed_props = Vec::new();
    for (id, stat_prop) in static_props {
        // We want to remove all properties of function symbols (parameters) not present in the BN.
        // These parameters were pruned beforehand because they are unused (redundant), and
        // we must do the same with their properties.

        // Everything else is currently encoded into first-order logic (into a "generic" property)
        match stat_prop.get_prop_data() {
            StatPropertyType::GenericStatProp(prop) => {
                let new_prop =
                    ProcessedStatProp::mk_fol(id.as_str(), prop.processed_formula.as_str());
                processed_props.push(new_prop);
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
                let new_prop = ProcessedStatProp::mk_fol(id.as_str(), &formula);
                processed_props.push(new_prop);
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
                let new_prop = ProcessedStatProp::mk_fol(id.as_str(), &formula);
                processed_props.push(new_prop);
            }
            StatPropertyType::FnInputEssential(prop)
            | StatPropertyType::FnInputEssentialContext(prop) => {
                let fn_id = prop.target.clone().unwrap();
                // Only process this property if the BN contains this function symbol as its valid
                // parameter (otherwise it was pruned out for being unused).
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
                    let new_prop = ProcessedStatProp::mk_fol(id.as_str(), &formula);
                    processed_props.push(new_prop);
                }
            }
            StatPropertyType::FnInputMonotonic(prop)
            | StatPropertyType::FnInputMonotonicContext(prop) => {
                let fn_id = prop.target.clone().unwrap();
                // Only process this property if the BN contains this function symbol as its valid
                // parameter (otherwise it was pruned out for being unused).
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
                    let new_prop = ProcessedStatProp::mk_fol(id.as_str(), &formula);
                    processed_props.push(new_prop);
                }
            }
        }
    }

    Ok(processed_props)
}
