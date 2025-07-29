use crate::sketchbook::ids::UninterpretedFnId;
use crate::sketchbook::model::{
    Essentiality, FnArgumentProperty, ModelState, Monotonicity, UninterpretedFn,
};
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Structure for sending data about `UninterpretedFn` to the frontend.
///
/// `UninterpretedFnData` does not have the exact same fields as `UninterpretedFn` (for instance, there
/// is an additional useful field `id`). Some fields of `UninterpretedFnData` are simplified to allow easier
/// (de)serialization and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UninterpretedFnData {
    pub id: String,
    pub name: String,
    pub annotation: String,
    pub arguments: Vec<(Monotonicity, Essentiality)>,
    pub expression: String,
}

impl JsonSerde<'_> for UninterpretedFnData {}

impl UninterpretedFnData {
    /// Create new `UninterpretedFnData` object given an uninterpreted fn's `name`, `arity`, and `id`.
    pub fn new(
        id: &str,
        name: &str,
        annot: &str,
        arg_properties: &[FnArgumentProperty],
        expression: &str,
    ) -> UninterpretedFnData {
        let arg_props_transformed = arg_properties
            .iter()
            .map(|a| (a.monotonicity, a.essential))
            .collect();
        UninterpretedFnData {
            id: id.to_string(),
            name: name.to_string(),
            annotation: annot.to_string(),
            arguments: arg_props_transformed,
            expression: expression.to_string(),
        }
    }

    /// Create new `UninterpretedFnData` object given an uninterpreted function and its id.
    pub fn from_fn(
        fn_id: &UninterpretedFnId,
        uninterpreted_fn: &UninterpretedFn,
    ) -> UninterpretedFnData {
        let arguments = uninterpreted_fn.get_all_arguments();
        UninterpretedFnData::new(
            fn_id.as_str(),
            uninterpreted_fn.get_name(),
            uninterpreted_fn.get_annotation(),
            arguments,
            uninterpreted_fn.get_fn_expression(),
        )
    }

    /// Extract new `UninterpretedFn` instance from this data (if the function's expression
    /// is valid).
    ///
    /// Model is given for validity check during parsing the function's expression.
    pub fn to_uninterpreted_fn(&self, model: &ModelState) -> Result<UninterpretedFn, String> {
        let arguments: Vec<FnArgumentProperty> = self
            .arguments
            .iter()
            .map(|(m, e)| FnArgumentProperty::new(*e, *m))
            .collect();
        let arity = arguments.len();
        let own_id = &model.get_uninterpreted_fn_id(&self.id)?;
        UninterpretedFn::new_default(&self.name, arity)?
            .with_annotation(&self.annotation)
            .with_argument_properties(arguments)?
            .with_expression(&self.expression, model, own_id)
    }
}
