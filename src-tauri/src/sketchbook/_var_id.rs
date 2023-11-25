use crate::sketchbook::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// A type-safe (string-based) identifier of a `Variable` inside `RegulationsState`.
///
/// **Warning:** Do not mix identifiers between different networks/graphs. Generally, be careful
/// to only use `VarIds` currently valid for the network.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct VarId {
    id: Identifier,
}

impl VarId {
    /// Construct new instances of `VarId`s from string.
    ///
    /// Returns `Err` if the string is not a valid identifier (it must be a C-like identifier).
    ///
    /// This does not ensure that the generated ID is unique and valid for given context.
    /// Parent classes (like `RegulationsState`) allow to generate unique `VarIds` safely.
    pub(crate) fn new(identifier: &str) -> Result<VarId, String> {
        Ok(VarId {
            id: Identifier::new(identifier)?,
        })
    }

    /// Access the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        self.id.as_str()
    }
}

impl Display for VarId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.id)
    }
}

impl FromStr for VarId {
    type Err = String;

    fn from_str(s: &str) -> Result<VarId, <VarId as FromStr>::Err> {
        VarId::new(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::VarId;
    use std::str::FromStr;

    #[test]
    fn test_valid_var_id() {
        let var_string = "variable";
        let var = VarId::new(var_string).unwrap();
        let same_var = VarId::from_str(var_string).unwrap();
        assert_eq!(var, same_var);

        assert_eq!(var.as_str(), var_string);
        assert_eq!(var.to_string(), var_string.to_string());
    }

    #[test]
    fn test_invalid_var_id() {
        let var_string = "invalid %%% id";
        let var = VarId::new(var_string);
        assert!(var.is_err());

        let var = VarId::from_str(var_string);
        assert!(var.is_err());
    }
}
