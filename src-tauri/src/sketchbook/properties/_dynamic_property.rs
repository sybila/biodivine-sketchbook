use crate::sketchbook::observations::{Dataset, Observation};
use crate::sketchbook::properties::_mk_formulas::*;
use serde::{Deserialize, Serialize};

/// A typesafe representation of a dynamic property expressed by a formula.
///
/// TODO: Currently, this is made considering only HCTL properties, but that will change.
/// TODO: That might result in some finer property hierarchy.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DynamicProperty {
    pub formula: String,
}

impl DynamicProperty {
    /// Create `DynamicProperty` object directly from a formula and id string slices.
    ///
    /// TODO: add syntax check.
    pub fn try_from_str(formula: &str) -> Result<DynamicProperty, String> {
        // todo: syntax check
        Ok(DynamicProperty::new_raw(formula))
    }

    /// **internal** Create `DynamicProperty` object directly from a string formula.
    ///
    /// Note that this does not perform any syntax checks.
    fn new_raw(formula: &str) -> Self {
        DynamicProperty {
            formula: formula.to_string(),
        }
    }

    /// Encode an observation by a (propositional) formula depicting the corresponding state/sub-space.
    /// The observation's binary values are used to create a conjunction of literals.
    /// The `var_names` are used as propositions names in the formula.
    pub fn encode_observation(
        obs: &Observation,
        var_names: &[String],
    ) -> Result<DynamicProperty, String> {
        let formula = encode_observation(obs, var_names)?;
        Ok(DynamicProperty::new_raw(&formula))
    }

    /// Encode each of the several observations, one by one.
    /// For details, see [Self::encode_observation].
    pub fn encode_multiple_observations(
        observations: &[Observation],
        var_names: &[String],
    ) -> Result<Vec<DynamicProperty>, String> {
        let formulae = encode_multiple_observations(observations, var_names)?;
        let properties = formulae
            .iter()
            .map(|f| DynamicProperty::new_raw(f))
            .collect();
        Ok(properties)
    }

    /// Encode a dataset of observations as a single HCTL formula. The particular formula
    /// template is chosen depending on the type of data (attractor data, time-series, ...).
    ///
    /// Only data with their type specified can be encoded.
    pub fn try_encode_observation_list_hctl(obs_list: &Dataset) -> Result<DynamicProperty, String> {
        let formula = encode_observation_list_hctl(obs_list)?;
        Ok(DynamicProperty::new_raw(&formula))
    }
}
