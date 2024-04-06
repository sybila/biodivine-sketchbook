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

/// **(internal)** A base class to derive type-safe identifiers from (using a macro below).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
struct BaseId {
    id: String,
}

impl BaseId {
    pub(crate) fn new(identifier: &str) -> Result<Self, String> {
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

    pub fn as_str(&self) -> &str {
        self.id.as_str()
    }
}

impl Display for BaseId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.id)
    }
}

impl FromStr for BaseId {
    type Err = String;

    fn from_str(s: &str) -> Result<BaseId, <BaseId as FromStr>::Err> {
        BaseId::new(s)
    }
}

/// Macro to define various kinds of type-safe identifiers based on `BaseId`.
macro_rules! id_wrapper {
    ($TypeName:ident, $doc:expr) => {
        #[doc = "A type-safe (string-based) identifier of a `"]
        #[doc = $doc]
        #[doc = "` inside a particular model."]
        #[doc = ""]
        #[doc = "**Warning:** Do not mix identifiers between different models/sketches."]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
        pub struct $TypeName(BaseId);

        impl $TypeName {
            /// Try to parse new identifier from a string.
            ///
            /// Return `Err` if the string is not a valid C-like identifier.
            ///
            /// This does not ensure that the generated ID is unique and usable for given context.
            pub fn new(id: &str) -> Result<Self, String> {
                BaseId::new(id).map($TypeName)
            }

            /// Access the identifier as a string slice.
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl std::fmt::Display for $TypeName {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl std::str::FromStr for $TypeName {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $TypeName::new(s)
            }
        }
    };
}

id_wrapper!(DatasetId, "Dataset");
id_wrapper!(DynPropertyId, "DynProperty");
id_wrapper!(LayoutId, "Layout");
id_wrapper!(ObservationId, "Observation");
id_wrapper!(StatPropertyId, "StatProperty");
id_wrapper!(UninterpretedFnId, "UninterpretedFn");
id_wrapper!(VarId, "Variable");

#[cfg(test)]
mod tests {
    use crate::sketchbook::ids::BaseId;
    use std::str::FromStr;

    #[test]
    fn test_valid_id() {
        let id_string = "id___";
        let id = BaseId::new(id_string).unwrap();
        let same_id = BaseId::from_str(id_string).unwrap();
        assert_eq!(id, same_id);

        assert_eq!(id.as_str(), id_string);
        assert_eq!(id.to_string(), id_string.to_string());
    }

    #[test]
    fn test_invalid_id() {
        let id_string = "invalid %%% id";
        let id = BaseId::new(id_string);
        assert!(id.is_err());

        let id = BaseId::from_str(id_string);
        assert!(id.is_err());
    }
}
