use crate::sketchbook::observations::{Dataset, Observation};
use crate::sketchbook::properties::_mk_hctl_formulas::*;
use serde::{Deserialize, Serialize};

/// A typesafe representation of a dynamic property expressed by a formula.
///
/// TODO: Currently, this is just a placeholder for HCTL properties, but that will probably change
/// TODO: once we introduce property templates. That might result in some finer property hierarchy.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DynProperty {
    formula: String,
}

/// Creating properties.
impl DynProperty {
    /// Create ` DynProperty` object directly from a formula, which must be in a correct format.
    ///
    /// TODO: add syntax check.
    pub fn try_from_str(formula: &str) -> Result<DynProperty, String> {
        // todo: syntax check
        Ok(DynProperty::new_raw(formula))
    }

    /// **internal** Create ` DynProperty` object directly from a string formula,
    /// without any syntax checks on it.
    fn new_raw(formula: &str) -> Self {
        DynProperty {
            formula: formula.to_string(),
        }
    }

    /// Encode an observation by a (propositional) formula depicting the corresponding state/sub-space.
    /// The observation's binary values are used to create a conjunction of literals.
    /// The `var_names` are used as propositions names in the formula.
    pub fn encode_observation(
        obs: &Observation,
        var_names: &[String],
    ) -> Result<DynProperty, String> {
        let formula = encode_observation(obs, var_names)?;
        Ok(DynProperty::new_raw(&formula))
    }

    /// Encode each of the several observations, one by one.
    /// For details, see [Self::encode_observation].
    pub fn encode_multiple_observations(
        observations: &[Observation],
        var_names: &[String],
    ) -> Result<Vec<DynProperty>, String> {
        let formulae = encode_multiple_observations(observations, var_names)?;
        let properties = formulae.iter().map(|f| DynProperty::new_raw(f)).collect();
        Ok(properties)
    }

    /// Encode a dataset of observations as a single HCTL formula. The particular formula
    /// template is chosen depending on the type of data (attractor data, time-series, ...).
    ///
    /// Only data with their type specified can be encoded.
    pub fn try_encode_dataset_hctl(dataset: &Dataset) -> Result<DynProperty, String> {
        let formula = encode_observation_list_hctl(dataset)?;
        Ok(DynProperty::new_raw(&formula))
    }
}

/// Editing properties.
impl DynProperty {
    // TODO
}

/// Observing properties.
impl DynProperty {
    pub fn get_formula(&self) -> &str {
        &self.formula
    }
}
