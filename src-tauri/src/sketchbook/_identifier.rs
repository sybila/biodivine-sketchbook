use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// **(internal)** A regex string of an identifier which we currently allow to appear.
const ID_REGEX_STR: &str = r"^[a-zA-Z_][a-zA-Z0-9_]*$";

lazy_static! {
    /// A regular expression that matches the identifiers allowed.
    static ref ID_REGEX: Regex = Regex::new(ID_REGEX_STR).unwrap();
}

/// A type-safe identifier that can be used for IDs of various objects, such as of variables
/// (see `VarId`) or layouts (see `LayoutId`). Corresponds to a C-like identifier, or SBML's SId.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Identifier {
    id: String,
}

impl Identifier {
    /// Construct new instances of `Identifier`s.
    ///
    /// Returns `Err` if the identifier is not a valid C-like identifier.
    pub fn new(identifier: &str) -> Result<Identifier, String> {
        if Self::is_valid_identifier(identifier) {
            Ok(Self {
                id: identifier.to_string(),
            })
        } else {
            Err(format!("Invalid identifier: {identifier}"))
        }
    }

    /// Check if the string is a valid (C-like) identifier.
    fn is_valid_identifier(s: &str) -> bool {
        ID_REGEX.is_match(s)
    }

    /// Access the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.id)
    }
}

impl FromStr for Identifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Identifier, <Identifier as FromStr>::Err> {
        Identifier::new(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::Identifier;
    use std::str::FromStr;

    #[test]
    fn test_valid_id() {
        let id_string = "id___";
        let id = Identifier::new(id_string).unwrap();
        let same_id = Identifier::from_str(id_string).unwrap();
        assert_eq!(id, same_id);

        assert_eq!(id.as_str(), id_string);
        assert_eq!(id.to_string(), id_string.to_string());
    }

    #[test]
    fn test_invalid_id() {
        let id_string = "invalid %%% id";
        let id = Identifier::new(id_string);
        assert!(id.is_err());

        let id = Identifier::from_str(id_string);
        assert!(id.is_err());
    }
}
