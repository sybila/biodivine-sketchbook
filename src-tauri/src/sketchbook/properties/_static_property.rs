use serde::{Deserialize, Serialize};

/// A typesafe representation of a static property expressed by a formula.
///
/// TODO: Currently, this is just a placeholder.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StatProperty {}

/// Creating properties.
impl StatProperty {
    /// TODO - just a placeholder for now
    pub fn new() -> StatProperty {
        StatProperty {}
    }
}

impl Default for StatProperty {
    fn default() -> Self {
        Self::new()
    }
}
