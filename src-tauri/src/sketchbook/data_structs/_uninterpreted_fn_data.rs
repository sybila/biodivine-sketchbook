use crate::sketchbook::{
    Essentiality, FnArgument, JsonSerde, Monotonicity, UninterpretedFn, UninterpretedFnId,
};
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
    pub arguments: Vec<(Monotonicity, Essentiality)>,
    pub expression: String,
}

impl<'de> JsonSerde<'de> for UninterpretedFnData {}

impl UninterpretedFnData {
    /// Create new `UninterpretedFnData` object given an uninterpreted fn's `name`, `arity`, and `id`.
    pub fn new(
        id: &str,
        name: &str,
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
            arguments,
            uninterpreted_fn.get_fn_expression(),
        )
    }
}
