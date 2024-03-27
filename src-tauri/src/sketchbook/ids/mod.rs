use serde::{Deserialize, Serialize};

/// **(internal)** Definition and methods for `BaseId`.
mod _base_id;
/// **(internal)** Definition and methods for `Identifier`.
mod _identifier;

use _base_id::BaseId;

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
            /// Try to construct new identifiers from string.
            ///
            /// Returns `Err` if the string is not a valid identifier (it must be a C-like identifier).
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

id_wrapper!(LayoutId, "Layout");
id_wrapper!(ObservationId, "Observation");
id_wrapper!(DatasetId, "Dataset");
id_wrapper!(PropertyId, "Property");
id_wrapper!(UninterpretedFnId, "UninterpretedFn");
id_wrapper!(VarId, "Variable");
