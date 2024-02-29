use crate::sketchbook::{Essentiality, Monotonicity, Regulation};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending simplified data about `Regulation` to the frontend.
///
/// Some fields of `RegulationData` are simplified compared to `Regulation` (e.g., pure `Strings` instead
/// of more complex typesafe structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegulationData {
    pub regulator: String,
    pub target: String,
    pub sign: Monotonicity,
    pub essential: Essentiality,
}

impl RegulationData {
    /// Create new `RegulationData` object given references to individual components.
    pub fn new(
        regulator_id: &str,
        target_id: &str,
        essential: Essentiality,
        sign: Monotonicity,
    ) -> RegulationData {
        RegulationData {
            regulator: regulator_id.to_string(),
            target: target_id.to_string(),
            essential,
            sign,
        }
    }

    /// Create new `RegulationData` object given a `regulation`.
    pub fn from_reg(regulation: &Regulation) -> RegulationData {
        RegulationData {
            regulator: regulation.get_regulator().to_string(),
            target: regulation.get_target().to_string(),
            essential: *regulation.get_essentiality(),
            sign: *regulation.get_sign(),
        }
    }

    /// Try to create new `RegulationData` object given a string encoding a regulation.
    pub fn try_from_reg_str(regulation_str: &str) -> Result<RegulationData, String> {
        let regulation = Regulation::try_from_string(regulation_str)?;
        Ok(RegulationData::from_reg(&regulation))
    }
}

impl Display for RegulationData {
    /// Use json serialization to convert `RegulationData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for RegulationData {
    type Err = String;

    /// Use json de-serialization to construct `RegulationData` from string.
    fn from_str(s: &str) -> Result<RegulationData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
