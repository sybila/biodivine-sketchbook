use crate::sketchbook::ids::UninterpretedFnId;
use crate::sketchbook::model::{
    Essentiality, FnArgument, ModelState, Monotonicity, UninterpretedFn,
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
        arguments: &[FnArgument],
        expression: &str,
    ) -> UninterpretedFnData {
        let arguments_transformed = arguments
            .iter()
            .map(|a| (a.monotonicity, a.essential))
            .collect();
        UninterpretedFnData {
            id: id.to_string(),
            name: name.to_string(),
            annotation: annot.to_string(),
            arguments: arguments_transformed,
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
        let arguments = self
            .arguments
            .iter()
            .map(|(m, e)| FnArgument::new(*e, *m))
            .collect();
        UninterpretedFn::new(
            &self.name,
            &self.annotation,
            &self.expression,
            arguments,
            model,
            &model.get_uninterpreted_fn_id(&self.id)?,
        )
    }
}
