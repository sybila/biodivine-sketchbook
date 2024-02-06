use crate::sketchbook::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// A type-safe (string-based) identifier of a `Parameter` of a `ModelState`.
///
/// **Warning:** Do not mix identifiers between different models.
/// Generally, be careful to only use `ParamId` that are currently valid for the network.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ParamId {
    id: Identifier,
}

impl ParamId {
    /// Construct new instances of `ParamId`s from a string.
    ///
    /// Returns `Err` if the string is not a valid identifier (it must be a C-like identifier).
    ///
    /// This does not ensure that the generated ID is unique and valid for given context.
    /// Parent classes (like `ModelState`) allow to generate unique `ParamId` safely.
    pub(crate) fn new(identifier: &str) -> Result<ParamId, String> {
        Ok(ParamId {
            id: Identifier::new(identifier)?,
        })
    }

    /// Access the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        self.id.as_str()
    }
}

impl Display for ParamId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.id)
    }
}

impl FromStr for ParamId {
    type Err = String;

    fn from_str(s: &str) -> Result<ParamId, <ParamId as FromStr>::Err> {
        ParamId::new(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::ParamId;
    use std::str::FromStr;

    #[test]
    fn test_valid_var_id() {
        let param_string = "param";
        let param = ParamId::new(param_string).unwrap();
        let same_param = ParamId::from_str(param_string).unwrap();
        assert_eq!(param, same_param);

        assert_eq!(param.as_str(), param_string);
        assert_eq!(param.to_string(), same_param.to_string());
    }

    #[test]
    fn test_invalid_var_id() {
        let param_string = "invalid %%% id";
        let param = ParamId::new(param_string);
        assert!(param.is_err());

        let param = ParamId::from_str(param_string);
        assert!(param.is_err());
    }
}
