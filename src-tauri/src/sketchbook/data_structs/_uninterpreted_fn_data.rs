use crate::sketchbook::{Essentiality, Monotonicity, UninterpretedFn, UninterpretedFnId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about `UninterpretedFn` to the frontend.
///
/// `UninterpretedFnData` does not have the exact same fields as `UninterpretedFn` (for instance, there
/// is an additional useful field `id`). All the fields of `UninterpretedFnData` are string to allow for simpler
/// (de)serialization and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UninterpretedFnData {
    pub id: String,
    pub name: String,
    pub arity: usize,
    pub essentialities: Vec<Essentiality>,
    pub monotonicities: Vec<Monotonicity>,
    pub expression: String,
}

impl UninterpretedFnData {
    /// Create new `UninterpretedFnData` object given a uninterpreted fn's `name`, `arity`, and `id`.
    pub fn new(
        id: &str,
        name: &str,
        arity: usize,
        essentialities: Vec<Essentiality>,
        monotonicities: Vec<Monotonicity>,
        expression: &str,
    ) -> UninterpretedFnData {
        UninterpretedFnData {
            id: id.to_string(),
            name: name.to_string(),
            arity,
            essentialities,
            monotonicities,
            expression: expression.to_string(),
        }
    }

    /// Create new `UninterpretedFnData` object given a `uninterpreted fn` and its id.
    pub fn from_uninterpreted_fn(
        fn_id: &UninterpretedFnId,
        uninterpreted_fn: &UninterpretedFn,
    ) -> UninterpretedFnData {
        UninterpretedFnData {
            id: fn_id.to_string(),
            name: uninterpreted_fn.get_name().to_string(),
            arity: uninterpreted_fn.get_arity(),
            essentialities: uninterpreted_fn.get_all_essential().clone(),
            monotonicities: uninterpreted_fn.get_all_monotonic().clone(),
            expression: uninterpreted_fn.get_fn_expression().to_string(),
        }
    }
}

impl Display for UninterpretedFnData {
    /// Use json serialization to convert `UninterpretedFnData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for UninterpretedFnData {
    type Err = String;

    /// Use json de-serialization to construct `UninterpretedFnData` from string.
    fn from_str(s: &str) -> Result<UninterpretedFnData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
