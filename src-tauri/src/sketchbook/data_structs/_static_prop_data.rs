use crate::sketchbook::ids::StatPropertyId;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::properties::StatProperty;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct GenericStatPropData {
    pub formula: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UpdateFnInputEssentialData {
    pub input: String,
    pub target: String,
    pub value: String,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FnInputEssentialData {
    pub input_index: usize,
    pub target: String,
    pub value: String,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UpdateFnInputMonotonicData {
    pub input: String,
    pub target: String,
    pub value: String,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FnInputMonotonicData {
    pub input_index: usize,
    pub target: String,
    pub value: String,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum StatPropertyTypeData {
    GenericStatProp(GenericStatPropData),
    UpdateFnInputEssential(UpdateFnInputEssentialData),
    FnInputEssential(FnInputEssentialData),
    UpdateFnInputMonotonic(UpdateFnInputMonotonicData),
    FnInputMonotonic(FnInputMonotonicData),
}

/// Structure for sending data about static properties to the frontend.
///
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatPropertyData {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    variant: StatPropertyTypeData,
}

impl<'de> JsonSerde<'de> for StatPropertyData {}

impl StatPropertyData {
    /// Create new generic `StatPropertyData` instance given a properties `id`, `name`, and `formula`.
    pub fn new_generic(id: &str, name: &str, formula: &str) -> StatPropertyData {
        StatPropertyData {
            id: id.to_string(),
            name: name.to_string(),
            variant: StatPropertyTypeData::GenericStatProp(GenericStatPropData {
                formula: formula.to_string(),
            }),
        }
    }

    /// Create new `StatPropertyData` object given a reference to a property and its `id`.
    /// TODO: placeholder, missing implementation for other types of properties
    pub fn from_property(id: &StatPropertyId, property: &StatProperty) -> StatPropertyData {
        let name = property.get_name();
        match property.get_prop_data() {
            StatPropertyType::GenericStatProp(p) => {
                Self::new_generic(id.as_str(), name, &p.raw_formula)
            }
            _ => todo!(),
        }
    }

    /// Extract the corresponding `StatProperty` instance from this `StatPropertyData`.
    /// TODO: placeholder, missing implementation for other types of properties
    pub fn to_property(&self) -> Result<StatProperty, String> {
        let name = self.name.as_str();
        match &self.variant {
            StatPropertyTypeData::GenericStatProp(p) => StatProperty::mk_generic(name, &p.formula),
            _ => todo!(),
        }
    }
}
