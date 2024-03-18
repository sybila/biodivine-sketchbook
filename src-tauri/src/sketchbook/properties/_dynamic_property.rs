use crate::sketchbook::observations::{Observation, ObservationList};
use crate::sketchbook::properties::_mk_formulas::*;
use serde::{Deserialize, Serialize};

/// A typesafe representation of a dynamic property expressed by a formula.
///
/// TODO: Currently, this is made considering only HCTL properties, but that will change.
/// TODO: That might result in some finer property hierarchy.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DynamicProperty {
    pub formula: String,
    pub id: String,
}

impl DynamicProperty {
    /// Create `DynamicProperty` object directly from a formula and id string slices.
    ///
    /// TODO: syntax check.
    pub fn try_from_str(formula: &str, id: &str) -> Result<DynamicProperty, String> {
        // todo: syntax check

        Ok(DynamicProperty::new_raw(formula, id))
    }

    /// **internal** Create `DynamicProperty` object directly from a formula and id string slices.
    fn new_raw(formula: &str, id: &str) -> Self {
        DynamicProperty {
            id: id.to_string(),
            formula: formula.to_string(),
        }
    }

    /// Encode an observation by a (propositional) formula depicting the corresponding state/sub-space.
    /// The observation's binary values are used to create a conjunction of literals.
    /// The `var_names` are used as propositions names in the formula.
    pub fn encode_observation(
        obs: &Observation,
        var_names: &[String],
        id: &str,
    ) -> Result<DynamicProperty, String> {
        let formula = encode_observation(obs, var_names)?;
        Ok(DynamicProperty::new_raw(&formula, id))
    }

    /// Encode each of the several observations, one by one.
    /// For details, see [Self::encode_observation].
    pub fn encode_multiple_observations(
        observations: &[Observation],
        var_names: &[String],
        ids: &[&str],
    ) -> Result<Vec<DynamicProperty>, String> {
        let formulae = encode_multiple_observations(observations, var_names)?;
        let properties = formulae
            .iter()
            .zip(ids)
            .map(|(f, i)| DynamicProperty::new_raw(f, i))
            .collect();
        Ok(properties)
    }

    /// Encode a dataset of observations as a single HCTL formula. The particular formula
    /// template is chosen depending on the type of data (attractor data, time-series, ...).
    ///
    /// Only data with their type specified can be encoded.
    pub fn try_encode_observation_list_hctl(
        obs_list: &ObservationList,
        id: &str,
    ) -> Result<DynamicProperty, String> {
        let formula = encode_observation_list_hctl(obs_list)?;
        Ok(DynamicProperty::new_raw(&formula, id))
    }
}
