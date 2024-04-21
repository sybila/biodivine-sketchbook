use crate::sketchbook::ids::{DynPropertyId, StatPropertyId};
use crate::sketchbook::properties::{DynProperty, StatProperty};
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Structure for sending data about dynamic properties to the frontend.
///
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
///
/// TODO: currently just a placeholder
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynPropertyData {
    pub id: String,
    pub formula: String,
}

/// Structure for sending data about static properties to the frontend.
///
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
///
/// TODO: currently just a placeholder
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatPropertyData {
    pub id: String,
    pub formula: String,
}

impl<'de> JsonSerde<'de> for DynPropertyData {}
impl<'de> JsonSerde<'de> for StatPropertyData {}

impl DynPropertyData {
    /// Create new `DynPropertyData` object given a properties `id` and `formula`.
    pub fn new(id: &str, formula: &str) -> DynPropertyData {
        DynPropertyData {
            id: id.to_string(),
            formula: formula.to_string(),
        }
    }

    /// Create new `DynPropertyData` object given a reference to a property and its `id`.
    pub fn from_property(id: &DynPropertyId, property: &DynProperty) -> DynPropertyData {
        DynPropertyData::new(id.as_str(), property.get_formula())
    }

    /// Extract the corresponding `DynProperty` instance from this `DynPropertyData`.
    pub fn to_property(&self) -> Result<DynProperty, String> {
        DynProperty::try_from_str(&self.formula)
    }
}

impl StatPropertyData {
    /// Create new `StatPropertyData` object given a properties `id` and `formula`.
    pub fn new(id: &str, formula: &str) -> StatPropertyData {
        StatPropertyData {
            id: id.to_string(),
            formula: formula.to_string(),
        }
    }

    /// Create new `StatPropertyData` object given a reference to a property and its `id`.
    pub fn from_property(id: &StatPropertyId, property: &StatProperty) -> StatPropertyData {
        StatPropertyData::new(id.as_str(), property.get_formula())
    }

    /// Extract the corresponding `StatProperty` instance from this `StatPropertyData`.
    pub fn to_property(&self) -> Result<StatProperty, String> {
        StatProperty::try_from_str(&self.formula)
    }
}
