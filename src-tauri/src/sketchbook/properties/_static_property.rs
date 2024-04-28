use serde::{Deserialize, Serialize};

/// A typesafe representation of a static property expressed by a formula.
///
/// TODO: Currently, this is just a placeholder.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StatProperty {
    formula: String,
}

/// Creating properties.
impl StatProperty {
    /// Create `StatProperty` object directly from a formula, which must be in a correct format.
    ///
    /// TODO: add syntax check.
    pub fn try_from_str(formula: &str) -> Result<StatProperty, String> {
        // todo: syntax check
        Ok(StatProperty::new_raw(formula))
    }

    /// **internal** Create `StatProperty` object directly from a string formula,
    /// without any syntax checks on it.
    fn new_raw(formula: &str) -> Self {
        StatProperty {
            formula: formula.to_string(),
        }
    }
}

/// Editing static properties.
impl StatProperty {
    // TODO
}

/// Observing static properties.
impl StatProperty {
    pub fn get_formula(&self) -> &str {
        &self.formula
    }
}
