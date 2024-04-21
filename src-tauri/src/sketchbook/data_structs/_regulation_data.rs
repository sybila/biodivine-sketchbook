use crate::sketchbook::ids::VarId;
use crate::sketchbook::model::{Essentiality, Monotonicity, Regulation};
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

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

impl<'de> JsonSerde<'de> for RegulationData {}

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
        RegulationData::new(
            regulation.get_regulator().as_str(),
            regulation.get_target().as_str(),
            *regulation.get_essentiality(),
            *regulation.get_sign(),
        )
    }

    /// Try to create new `RegulationData` object given a string encoding a regulation.
    pub fn try_from_reg_str(regulation_str: &str) -> Result<RegulationData, String> {
        let regulation = Regulation::try_from_string(regulation_str)?;
        Ok(RegulationData::from_reg(&regulation))
    }

    /// Extract new `Regulation` instance from this data.
    pub fn to_reg(&self) -> Result<Regulation, String> {
        Ok(Regulation::new(
            VarId::new(&self.regulator)?,
            VarId::new(&self.target)?,
            self.essential,
            self.sign,
        ))
    }
}
