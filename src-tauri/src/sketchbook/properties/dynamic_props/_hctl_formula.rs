use crate::sketchbook::model::ModelState;
use crate::sketchbook::observations::{Dataset, Observation};
use crate::sketchbook::properties::dynamic_props::_mk_hctl_formulas::*;
use serde::{Deserialize, Serialize};

/// A typesafe representation of a HCTL formula used in dynamic properties.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct HctlFormula {
    formula: String,
    // todo: add tree representation
}

/// Creating hctl formulas.
impl HctlFormula {
    /// Parse `HctlFormula` object directly from a string, which must be in a correct format.
    ///
    /// TODO: add syntax check.
    pub fn try_from_str(formula: &str) -> Result<HctlFormula, String> {
        // todo: syntax check
        Ok(HctlFormula::new_raw(formula))
    }

    /// **internal** Create `HctlFormula` object directly from a string formula,
    /// without any syntax checks on it.
    fn new_raw(formula: &str) -> Self {
        HctlFormula {
            formula: formula.to_string(),
        }
    }

    /// Encode an observation by a (propositional) formula depicting the corresponding state/sub-space.
    /// The observation's binary values are used to create a conjunction of literals.
    /// The `var_names` are used as propositions names in the formula.
    pub fn encode_observation(
        obs: &Observation,
        var_names: &[String],
    ) -> Result<HctlFormula, String> {
        let formula = encode_observation(obs, var_names)?;
        Ok(HctlFormula::new_raw(&formula))
    }

    /// Encode each of the several observations, one by one.
    /// For details, see [Self::encode_observation].
    pub fn encode_multiple_observations(
        observations: &[Observation],
        var_names: &[String],
    ) -> Result<Vec<HctlFormula>, String> {
        let formulae = encode_multiple_observations(observations, var_names)?;
        let properties = formulae.iter().map(|f| HctlFormula::new_raw(f)).collect();
        Ok(properties)
    }

    /// Encode a dataset of observations as a single HCTL formula. The particular formula
    /// template is chosen depending on the type of data (attractor data, time-series, ...).
    ///
    /// Only data with their type specified can be encoded.
    pub fn try_encode_dataset_hctl(dataset: &Dataset) -> Result<HctlFormula, String> {
        let formula = encode_dataset_hctl(dataset)?;
        Ok(HctlFormula::new_raw(&formula))
    }
}

/// Editing HCTL formulas.
impl HctlFormula {
    // TODO
}

/// Observing HCTL formulas.
impl HctlFormula {
    /// Raw str version of the HCTL formula.
    pub fn as_str(&self) -> &str {
        &self.formula
    }
}

/// Static methods (to check validity of formula strings).
impl HctlFormula {
    /// Check if the formula is correctly formed based on HCTL syntactic rules.
    pub fn check_pure_syntax(formula: String) -> Result<(), String> {
        println!("For now, {formula} cannot be checked.");
        todo!()
    }

    /// Check if the formula is correctly formed based on HCTL syntactic rules, and also
    /// whether the propositions correspond to valid network variables used in the `model`.
    pub fn check_syntax_with_model(formula: String, model: &ModelState) -> Result<(), String> {
        println!("For now, {formula} cannot be checked with respect to the {model:?}.");
        todo!()
    }
}
