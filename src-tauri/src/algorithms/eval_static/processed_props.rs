use crate::algorithms::eval_static::encode::*;
use crate::sketchbook::properties::static_props::{StatProperty, StatPropertyType};
use crate::sketchbook::Sketch;

use biodivine_lib_param_bn::BooleanNetwork;

/// Processed static property encoded in FOL.
///
/// If we ever need more variants than just FOL prop, make this enum (like
/// the one used for dynamic props).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedStatProp {
    pub formula: String,
}

/// Simplified constructors for processed dynamic properties.
impl ProcessedStatProp {
    /// Create FOL `ProcessedStatProp` instance.
    pub fn mk_fol(formula: &str) -> ProcessedStatProp {
        ProcessedStatProp {
            formula: formula.to_string(),
        }
    }
}

pub fn process_static_props(
    sketch: &Sketch,
    bn: &BooleanNetwork,
) -> Result<Vec<ProcessedStatProp>, String> {
    let mut static_props = sketch.properties.stat_props().collect::<Vec<_>>();
    // sort properties by IDs for deterministic computation times (and get rid of the IDs)
    static_props.sort_by(|(a_id, _), (b_id, _)| a_id.cmp(b_id));
    let static_props: Vec<StatProperty> = static_props
        .into_iter()
        .map(|(_, prop)| prop.clone())
        .collect();

    let mut processed_props = Vec::new();
    for stat_prop in static_props {
        // currently, everything is encoded into first-order logic (into a "generic" property)
        let stat_prop_processed = match stat_prop.get_prop_data() {
            StatPropertyType::GenericStatProp(prop) => {
                ProcessedStatProp::mk_fol(prop.processed_formula.as_str())
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
                ProcessedStatProp::mk_fol(&formula)
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
                ProcessedStatProp::mk_fol(&formula)
            }
            StatPropertyType::FnInputEssential(prop)
            | StatPropertyType::FnInputEssentialContext(prop) => {
                let fn_id = prop.target.clone().unwrap();
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
                ProcessedStatProp::mk_fol(&formula)
            }
            StatPropertyType::FnInputMonotonic(prop)
            | StatPropertyType::FnInputMonotonicContext(prop) => {
                let fn_id = prop.target.clone().unwrap();
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
                ProcessedStatProp::mk_fol(&formula)
            }
        };
        processed_props.push(stat_prop_processed)
    }

    Ok(processed_props)
}
