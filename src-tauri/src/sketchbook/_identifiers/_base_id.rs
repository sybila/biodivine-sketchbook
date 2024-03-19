use crate::sketchbook::_identifiers::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// A base class to derive type-safe identifiers.
///
/// **Warning:** Do not mix identifiers between different models.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BaseId {
    id: Identifier,
}

impl BaseId {
    pub(crate) fn new(id: &str) -> Result<Self, String> {
        Ok(BaseId {
            id: Identifier::new(id)?,
        })
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
