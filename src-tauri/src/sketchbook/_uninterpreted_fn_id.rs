use crate::sketchbook::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// A type-safe (string-based) identifier of a `UninterpretedFn` of a `ModelState`.
///
/// **Warning:** Do not mix identifiers between different models.
/// Generally, be careful to only use `UninterpretedFnId` that are currently valid for the network.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct UninterpretedFnId {
    id: Identifier,
}

impl UninterpretedFnId {
    /// Construct new instances of `UninterpretedFnId`s from a string.
    ///
    /// Returns `Err` if the string is not a valid identifier (it must be a C-like identifier).
    ///
    /// This does not ensure that the generated ID is unique and valid for given context.
    /// Parent classes (like `ModelState`) allow to generate unique `UninterpretedFnId` safely.
    pub(crate) fn new(identifier: &str) -> Result<UninterpretedFnId, String> {
        Ok(UninterpretedFnId {
            id: Identifier::new(identifier)?,
        })
    }

    /// Access the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        self.id.as_str()
    }
}

impl Display for UninterpretedFnId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.id)
    }
}

impl FromStr for UninterpretedFnId {
    type Err = String;

    fn from_str(s: &str) -> Result<UninterpretedFnId, <UninterpretedFnId as FromStr>::Err> {
        UninterpretedFnId::new(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::UninterpretedFnId;
    use std::str::FromStr;

    #[test]
    fn test_valid_var_id() {
        let uninterpreted_fn_string = "uninterpreted_fn";
        let uninterpreted_fn = UninterpretedFnId::new(uninterpreted_fn_string).unwrap();
        let same_uninterpreted_fn = UninterpretedFnId::from_str(uninterpreted_fn_string).unwrap();
        assert_eq!(uninterpreted_fn, same_uninterpreted_fn);

        assert_eq!(uninterpreted_fn.as_str(), uninterpreted_fn_string);
        assert_eq!(
            uninterpreted_fn.to_string(),
            same_uninterpreted_fn.to_string()
        );
    }

    #[test]
    fn test_invalid_var_id() {
        let fn_string = "invalid %%% id";
        let uninterpreted_fn = UninterpretedFnId::new(fn_string);
        assert!(uninterpreted_fn.is_err());

        let uninterpreted_fn = UninterpretedFnId::from_str(fn_string);
        assert!(uninterpreted_fn.is_err());
    }
}
