use crate::algorithms::eval_dynamic::encode::encode_dataset_hctl_str;
use crate::sketchbook::observations::Dataset;
use crate::sketchbook::properties::dynamic_props::{DynProperty, DynPropertyType};
use crate::sketchbook::Sketch;

/// Enum of possible variants of data to encode.
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum DataEncodingType {
    Attractor,
    FixedPoint,
    TrapSpace,
}

/// Property requiring that a particular HCTL formula is satisfied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedHctlFormula {
    pub formula: String,
}

/// Property requiring that observations in a particular dataset are trap spaces.
/// The trap space might be required to be `minimal` or `non-percolable`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedTrapSpace {
    pub dataset: Dataset,
    pub minimal: bool,
    pub nonpercolable: bool,
}

/// Property requiring that the number of attractors falls into the range <minimal, maximal>.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedAttrCount {
    pub minimal: usize,
    pub maximal: usize,
}

/// Enum for processed variants of dynamic properties.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessedDynProp {
    ProcessedAttrCount(ProcessedAttrCount),
    ProcessedTrapSpace(ProcessedTrapSpace),
    ProcessedHctlFormula(ProcessedHctlFormula),
}

/// Simplified constructors for processed dynamic properties.
impl ProcessedDynProp {
    /// Create HCTL `ProcessedDynProp` instance.
    pub fn mk_hctl(formula: &str) -> ProcessedDynProp {
        let property = ProcessedHctlFormula {
            formula: formula.to_string(),
        };
        ProcessedDynProp::ProcessedHctlFormula(property)
    }

    /// Create trap-space `ProcessedDynProp` instance.
    /// To encode single observation, make a singleton dataset.
    pub fn mk_trap_space(dataset: Dataset, minimal: bool, nonpercolable: bool) -> ProcessedDynProp {
        let property = ProcessedTrapSpace {
            dataset,
            minimal,
            nonpercolable,
        };
        ProcessedDynProp::ProcessedTrapSpace(property)
    }

    /// Create attractor-count `ProcessedDynProp` instance.
    pub fn mk_attr_count(minimal: usize, maximal: usize) -> ProcessedDynProp {
        let property = ProcessedAttrCount { minimal, maximal };
        ProcessedDynProp::ProcessedAttrCount(property)
    }
}

pub fn process_dynamic_props(sketch: &Sketch) -> Result<Vec<ProcessedDynProp>, String> {
    let mut dynamic_props = sketch.properties.dyn_props().collect::<Vec<_>>();
    // sort properties by IDs for deterministic computation times (and get rid of the IDs)
    dynamic_props.sort_by(|(a_id, _), (b_id, _)| a_id.cmp(b_id));
    let dynamic_props: Vec<DynProperty> = dynamic_props
        .into_iter()
        .map(|(_, prop)| prop.clone())
        .collect();

    let mut processed_props = Vec::new();
    for dyn_prop in dynamic_props {
        // TODO: currently, some types of properties (like time-series) are still not implemented

        // we translate as many types of properties into HCTL, but we also treat some
        // as special cases (these will have their own optimized evaluation)

        let dyn_prop_processed = match dyn_prop.get_prop_data() {
            // handled as a special case
            DynPropertyType::AttractorCount(prop) => {
                ProcessedDynProp::mk_attr_count(prop.minimal, prop.maximal)
            }
            // handled as a special case
            DynPropertyType::ExistsTrapSpace(prop) => {
                let dataset_id = prop.dataset.clone().unwrap();
                let mut dataset = sketch.observations.get_dataset(&dataset_id)?.clone();

                // if we only want to encode single observation, lets restrict the dataset
                if let Some(obs_id) = &prop.observation {
                    let observation = dataset.get_obs(obs_id)?.clone();
                    let var_names = dataset.variable_names();
                    let var_names_ref = var_names.iter().map(|v| v.as_str()).collect();
                    dataset = Dataset::new(vec![observation], var_names_ref)?;
                }

                ProcessedDynProp::mk_trap_space(dataset, prop.minimal, prop.nonpercolable)
            }
            // default generic HCTL
            DynPropertyType::GenericDynProp(prop) => {
                ProcessedDynProp::mk_hctl(prop.processed_formula.as_str())
            }
            // translate to generic HCTL
            DynPropertyType::ExistsFixedPoint(prop) => {
                // TODO: maybe encode as multiple formulae if we have more than one observation (instead of a conjunction)?
                let dataset_id = prop.dataset.clone().unwrap();
                let dataset = sketch.observations.get_dataset(&dataset_id)?;
                let formula = encode_dataset_hctl_str(
                    dataset,
                    prop.observation.clone(),
                    DataEncodingType::FixedPoint,
                )?;
                ProcessedDynProp::mk_hctl(&formula)
            }
            // translate to generic HCTL
            DynPropertyType::HasAttractor(prop) => {
                // TODO: maybe encode as multiple formulae if we have more than one observation (instead of a conjunction)?
                let dataset_id = prop.dataset.clone().unwrap();
                let dataset = sketch.observations.get_dataset(&dataset_id)?;
                let formula = encode_dataset_hctl_str(
                    dataset,
                    prop.observation.clone(),
                    DataEncodingType::Attractor,
                )?;
                ProcessedDynProp::mk_hctl(&formula)
            }
            // TODO: finish handling of time-series
            DynPropertyType::ExistsTrajectory(..) => todo!(),
        };
        processed_props.push(dyn_prop_processed);
    }

    Ok(processed_props)
}
