use crate::algorithms::eval_dynamic::encode::encode_dataset_hctl_str;
use crate::sketchbook::observations::{Dataset, Observation};
use crate::sketchbook::properties::dynamic_props::{
    DynPropertyType, WildCardProposition, WildCardType,
};
use crate::sketchbook::Sketch;

/// Enum of possible variants of data encodings via HCTL.
/// This is used as arguments for HCTL encoding functions.
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum DataEncodingType {
    Attractor,
    FixedPoint,
    TrapSpace,
    TimeSeries,
}

/// Property requiring that a particular extended HCTL formula is satisfied.
///
/// The language of dynamic properties allows to write template sub-properties
/// using wild-card propositions. These wild card propositions are turned into
/// `sub_properties` field.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedHctlFormula {
    pub id: String,
    pub formula: String,
    pub sub_properties: Vec<ProcessedDynProp>,
}

/// A special case - this is not a full property, but just a template that
/// can be used as a sub-property in HCTL props. It should be taken as just
/// a simple shortcut, nothing more.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedObservation {
    pub id: String,
    pub obs: Observation,
    pub var_names: Vec<String>,
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
    /// This one is just for sub-properties.
    ProcessedObservation(ProcessedObservation),
}

/// Simplified constructors for processed dynamic properties.
impl ProcessedDynProp {
    /// Create HCTL `ProcessedDynProp` instance.
    ///
    /// Arg `processed_wild_cards` represents template sub-properties that were
    /// extracted from wild-card propositions in the formula.
    pub fn mk_hctl(
        id: &str,
        formula: &str,
        processed_wild_cards: Vec<ProcessedDynProp>,
    ) -> ProcessedDynProp {
        let property = ProcessedHctlFormula {
            id: id.to_string(),
            formula: formula.to_string(),
            sub_properties: processed_wild_cards,
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

    /// Create observation `ProcessedDynProp` instance.
    pub fn mk_obs(id: &str, obs: Observation, var_names: Vec<String>) -> ProcessedDynProp {
        let property = ProcessedObservation {
            id: id.to_string(),
            obs,
            var_names,
        };
        ProcessedDynProp::ProcessedObservation(property)
    }

    /// Get ID of the underlying processed property.
    pub fn id(&self) -> &str {
        match &self {
            ProcessedDynProp::ProcessedHctlFormula(prop) => &prop.id,
            ProcessedDynProp::ProcessedAttrCount(prop) => &prop.id,
            ProcessedDynProp::ProcessedTrapSpace(prop) => &prop.id,
            ProcessedDynProp::ProcessedSimpleTrajectory(prop) => &prop.id,
            ProcessedDynProp::ProcessedObservation(prop) => &prop.id,
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
                let sub_properties = process_wild_cards(&prop.wild_cards, sketch)?;
                ProcessedDynProp::mk_hctl(
                    id.as_str(),
                    prop.processed_formula.as_str(),
                    sub_properties,
                )
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
                ProcessedDynProp::mk_hctl(id.as_str(), &formula, Vec::new()) // no wild-cards
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
                ProcessedDynProp::mk_hctl(id.as_str(), &formula, Vec::new()) // no wild-cards
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
                    ProcessedDynProp::mk_hctl(id.as_str(), &formula, Vec::new())
                    // no wild-cards
                }
            }
        };
        processed_props.push(dyn_prop_processed);
    }

    Ok(processed_props)
}

/// Process special template wild-card propositions (from a single HCTL formula), turning them into
/// type-safe sub-properties. Each sub-property is encoded as `ProcessedDynProp` variant.
pub fn process_wild_cards(
    wild_cards: &Vec<WildCardProposition>,
    sketch: &Sketch,
) -> Result<Vec<ProcessedDynProp>, String> {
    let mut processed_props = Vec::new();
    for wild_card_prop in wild_cards {
        // the ID always corresponds to the "processed string" that appears in the formula
        let id = wild_card_prop.processed_string();

        let dyn_prop_processed = match wild_card_prop.get_prop_data() {
            WildCardType::Observation(data_id, obs_id) => {
                let dataset = sketch.observations.get_dataset(data_id)?;
                let observation = dataset.get_obs(obs_id)?;
                let var_names = dataset.variable_names();
                ProcessedDynProp::mk_obs(&id, observation.clone(), var_names)
            }
        };
        processed_props.push(dyn_prop_processed);
    }

    Ok(processed_props)
}
