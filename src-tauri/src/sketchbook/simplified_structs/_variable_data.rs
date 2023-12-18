use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Structure for sending simplified data about `Variable` to frontend.
/// Only contains some fields, in string format, to allow for simpler parsing and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableData {
    pub id: String,
    pub name: String,
}

impl VariableData {
    pub fn new(id: String, name: String) -> VariableData {
        VariableData { id, name }
    }
}

impl Display for VariableData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
