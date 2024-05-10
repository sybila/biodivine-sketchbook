use crate::sketchbook::model::ModelState;
use serde::{Deserialize, Serialize};

/// A typesafe representation of a first-order formula used in static properties.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FirstOrderFormula {
    formula: String,
    // todo: add tree representation
}

/// Creating first-order formulas.
impl FirstOrderFormula {
    /// Parse `FirstOrderFormula` instance directly from a string, which must be in a
    /// correct format.
    ///
    /// TODO: add syntax check.
    pub fn try_from_str(formula: &str) -> Result<FirstOrderFormula, String> {
        // todo: syntax check
        Ok(FirstOrderFormula::new_raw(formula))
    }

    /// **internal** Create `HctlFormula` object directly from a string formula,
    /// without any syntax checks on it.
    fn new_raw(formula: &str) -> Self {
        FirstOrderFormula {
            formula: formula.to_string(),
        }
    }
}

/// Editing first-order formulas.
impl FirstOrderFormula {
    // TODO
}

/// Observing first-order formulas.
impl FirstOrderFormula {
    /// Raw str version of the first-order formula.
    pub fn as_str(&self) -> &str {
        &self.formula
    }
}

/// Static methods (to check validity of formula strings).
impl FirstOrderFormula {
    /// Check if the formula is correctly formed based on predefined FO syntactic rules.
    pub fn check_pure_syntax(formula: String) -> Result<(), String> {
        println!("For now, {formula} cannot be checked.");
        todo!()
    }

    /// Check if the formula is correctly formed based on predefined FO syntactic rules, and also
    /// whether formula respects the model (propositions must be valid entities in the model and
    /// so on).
    pub fn check_syntax_with_model(formula: String, model: &ModelState) -> Result<(), String> {
        println!("For now, {formula} cannot be checked with respect to the {model:?}.");
        todo!()
    }
}
