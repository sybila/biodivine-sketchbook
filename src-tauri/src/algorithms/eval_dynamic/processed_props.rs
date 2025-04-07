use crate::algorithms::eval_dynamic::encode::encode_dataset_hctl_str;
use crate::sketchbook::observations::Dataset;
use crate::sketchbook::properties::dynamic_props::DynPropertyType;
use crate::sketchbook::Sketch;

/// Enum of possible variants of data encodings via HCTL.
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum DataEncodingType {
    Attractor,
    FixedPoint,
    TrapSpace,
    TimeSeries,
}

/// Property requiring that a particular HCTL formula is satisfied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedHctlFormula {
    pub id: String,
    pub formula: String,
}

/// Property requiring that observations in a particular dataset are trap spaces.
/// The trap space might be required to be `minimal` or `non-percolable`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedTrapSpace {
    pub id: String,
    pub dataset: Dataset,
    pub minimal: bool,
    pub nonpercolable: bool,
}

/// Property requiring that the number of attractors falls into the range <minimal, maximal>.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedAttrCount {
    pub id: String,
    pub minimal: usize,
    pub maximal: usize,
}

/// Property requiring existence of trajectory between fully specified observations (each observation
/// corresponds to a single state of the STG).
///
/// This is a special case of the trajectory property, and it can be evaluated more efficiently than
/// by using the HCTL model checker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedSimpleTrajectory {
    pub id: String,
    pub dataset: Dataset,
}

/// Enum for processed variants of dynamic properties.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessedDynProp {
    ProcessedAttrCount(ProcessedAttrCount),
    ProcessedTrapSpace(ProcessedTrapSpace),
    ProcessedHctlFormula(ProcessedHctlFormula),
    ProcessedSimpleTrajectory(ProcessedSimpleTrajectory),
}

/// Simplified constructors for processed dynamic properties.
impl ProcessedDynProp {
    /// Create HCTL `ProcessedDynProp` instance.
    pub fn mk_hctl(id: &str, formula: &str) -> ProcessedDynProp {
        let property = ProcessedHctlFormula {
            id: id.to_string(),
            formula: formula.to_string(),
        };
        ProcessedDynProp::ProcessedHctlFormula(property)
    }

    /// Create trap-space `ProcessedDynProp` instance.
    /// To encode single observation, make a singleton dataset.
    pub fn mk_trap_space(
        id: &str,
        dataset: Dataset,
        minimal: bool,
        nonpercolable: bool,
    ) -> ProcessedDynProp {
        let property = ProcessedTrapSpace {
            id: id.to_string(),
            dataset,
            minimal,
            nonpercolable,
        };
        ProcessedDynProp::ProcessedTrapSpace(property)
    }

    /// Create attractor-count `ProcessedDynProp` instance.
    pub fn mk_attr_count(id: &str, minimal: usize, maximal: usize) -> ProcessedDynProp {
        let property = ProcessedAttrCount {
            id: id.to_string(),
            minimal,
            maximal,
        };
        ProcessedDynProp::ProcessedAttrCount(property)
    }

    /// Create simple trajectory `ProcessedDynProp` instance.
    ///
    /// The given dataset must contain fully specified observations only, otherwise this type of
    /// property cant be used.
    pub fn mk_simple_trajectory(id: &str, dataset: Dataset) -> Result<ProcessedDynProp, String> {
        for obs in dataset.observations() {
            if obs.num_unspecified_values() > 0 {
                return Err(format!("Property {id} cant be transformed into simple trajectory, the dataset contains some missing values."));
            }
        }
        let property = ProcessedSimpleTrajectory {
            id: id.to_string(),
            dataset,
        };
        Ok(ProcessedDynProp::ProcessedSimpleTrajectory(property))
    }

    /// Get ID of the underlying processed property.
    pub fn id(&self) -> &str {
        match &self {
            ProcessedDynProp::ProcessedHctlFormula(prop) => &prop.id,
            ProcessedDynProp::ProcessedAttrCount(prop) => &prop.id,
            ProcessedDynProp::ProcessedTrapSpace(prop) => &prop.id,
            ProcessedDynProp::ProcessedSimpleTrajectory(prop) => &prop.id,
        }
    }
}

/// Process dynamic properties in a sketch, converting them into one of the supported
/// `ProcessedDynProp` variants. That usually means encoding them into HCTL, or doing
/// some other preprocessing.
pub fn process_dynamic_props(sketch: &Sketch) -> Result<Vec<ProcessedDynProp>, String> {
    let mut dynamic_props = sketch.properties.dyn_props().collect::<Vec<_>>();
    // sort properties by IDs for deterministic computation times (and get rid of the IDs)
    dynamic_props.sort_by(|(a_id, _), (b_id, _)| a_id.cmp(b_id));

    let mut processed_props = Vec::new();
    for (id, dyn_prop) in dynamic_props {
        // we translate as many types of properties into HCTL, but we also treat some
        // as special cases (these will have their own optimized evaluation)

        let dyn_prop_processed = match dyn_prop.get_prop_data() {
            // handled as a special case
            DynPropertyType::AttractorCount(prop) => {
                ProcessedDynProp::mk_attr_count(id.as_str(), prop.minimal, prop.maximal)
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
                    dataset = Dataset::new("trap_space_data", vec![observation], var_names_ref)?;
                }

                ProcessedDynProp::mk_trap_space(
                    id.as_str(),
                    dataset,
                    prop.minimal,
                    prop.nonpercolable,
                )
            }
            // default generic HCTL
            DynPropertyType::GenericDynProp(prop) => {
                ProcessedDynProp::mk_hctl(id.as_str(), prop.processed_formula.as_str())
            }
            // encode fixed-points HCTL formula
            DynPropertyType::ExistsFixedPoint(prop) => {
                // TODO: if we have whole dataset, instead of using conjunction, try encoding as multiple properties
                let dataset_id = prop.dataset.clone().unwrap();
                let dataset = sketch.observations.get_dataset(&dataset_id)?;
                let formula = encode_dataset_hctl_str(
                    dataset,
                    prop.observation.clone(),
                    DataEncodingType::FixedPoint,
                )?;
                ProcessedDynProp::mk_hctl(id.as_str(), &formula)
            }
            // encode attractors with HCTL formula
            DynPropertyType::HasAttractor(prop) => {
                // TODO: if we have whole dataset, instead of using conjunction, try encoding as multiple properties
                let dataset_id = prop.dataset.clone().unwrap();
                let dataset = sketch.observations.get_dataset(&dataset_id)?;
                let formula = encode_dataset_hctl_str(
                    dataset,
                    prop.observation.clone(),
                    DataEncodingType::Attractor,
                )?;
                ProcessedDynProp::mk_hctl(id.as_str(), &formula)
            }
            // encode time series with HCTL formula
            DynPropertyType::ExistsTrajectory(prop) => {
                let dataset_id = prop.dataset.clone().unwrap();
                let dataset = sketch.observations.get_dataset(&dataset_id)?;

                // if the dataset does not have any missing values and has at least 3 observations, we
                // use an optimized reachability-based method
                // otherwise, a standard HCTL formula is created and later evaluated with model checker

                let no_missing_values = dataset
                    .observations()
                    .iter()
                    .all(|obs| (obs.num_unspecified_values() == 0));
                if no_missing_values && dataset.num_observations() > 2 {
                    // we can unwrap, since we checked no values are missing
                    ProcessedDynProp::mk_simple_trajectory(id.as_str(), dataset.clone()).unwrap()
                } else {
                    // TODO: also optimize the computation for the base case to avoid pure model checking
                    let formula =
                        encode_dataset_hctl_str(dataset, None, DataEncodingType::TimeSeries)?;
                    ProcessedDynProp::mk_hctl(id.as_str(), &formula)
                }
            }
        };
        processed_props.push(dyn_prop_processed);
    }

    Ok(processed_props)
}
