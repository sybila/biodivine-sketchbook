use crate::sketchbook::data_structs::_layout_node_data::LayoutNodeDataPrototype;
use crate::sketchbook::ids::VarId;
use crate::sketchbook::model::{UpdateFn, Variable};
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Structure for sending data about `Variable` and its `UpdateFn` to the frontend.
///
/// `VariableData` contains similar fields as `Variable` and additional fields `id` and `update_fn`.
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableData {
    pub id: String,
    pub name: String,
    pub annotation: String,
    pub update_fn: String,
}

/// Structure for receiving data about `Variable` and all of its `Layout` data from the frontend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableWithLayoutData {
    pub variable: VariableData,
    pub layouts: Vec<LayoutNodeDataPrototype>,
}

impl JsonSerde<'_> for VariableData {}
impl JsonSerde<'_> for VariableWithLayoutData {}

impl VariableData {
    /// Create new `VariableData` object given a variable's `name` and `id` string slices.
    pub fn new(id: &str, name: &str, annotation: &str, update_fn: &str) -> VariableData {
        VariableData {
            id: id.to_string(),
            name: name.to_string(),
            annotation: annotation.to_string(),
            update_fn: update_fn.to_string(),
        }
    }

    /// Create new `VariableData` object given a reference to a variable, its update function,
    /// and its id.
    pub fn from_var(var_id: &VarId, variable: &Variable, update_fn: &UpdateFn) -> VariableData {
        VariableData::new(
            var_id.as_str(),
            variable.get_name(),
            variable.get_annotation(),
            update_fn.get_fn_expression(),
        )
    }

    /// Extract new `Variable` instance from this data.
    pub fn to_var(&self) -> Result<Variable, String> {
        Variable::new_annotated(&self.name, &self.annotation)
    }
}
