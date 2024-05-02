use crate::sketchbook::ids::{StatPropertyId, UninterpretedFnId, VarId};
use crate::sketchbook::model::{Essentiality, Monotonicity};
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
    pub value: Essentiality,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FnInputEssentialData {
    pub input_index: usize,
    pub target: String,
    pub value: Essentiality,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UpdateFnInputMonotonicData {
    pub input: String,
    pub target: String,
    pub value: Monotonicity,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FnInputMonotonicData {
    pub input_index: usize,
    pub target: String,
    pub value: Monotonicity,
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
    /// Shorthand to create new generic `StatPropertyData` instance given a properties
    /// `id`, `name`, and `formula`.
    pub fn new_generic(id: &str, name: &str, formula: &str) -> StatPropertyData {
        let variant = StatPropertyTypeData::GenericStatProp(GenericStatPropData {
            formula: formula.to_string(),
        });
        Self::new_raw(id, name, variant)
    }

    /// Create new `StatPropertyData` object given a reference to a property and its `id`.
    pub fn from_property(id: &StatPropertyId, property: &StatProperty) -> StatPropertyData {
        let name = property.get_name();
        let variant = match property.get_prop_data() {
            StatPropertyType::GenericStatProp(p) => {
                StatPropertyTypeData::GenericStatProp(GenericStatPropData {
                    formula: p.raw_formula.clone(),
                })
            }
            StatPropertyType::UpdateFnInputEssential(p) => {
                StatPropertyTypeData::UpdateFnInputEssential(UpdateFnInputEssentialData {
                    input: p.input.to_string(),
                    target: p.target.to_string(),
                    value: p.value,
                    context: p.context.clone(),
                })
            }
            StatPropertyType::UpdateFnInputMonotonic(p) => {
                StatPropertyTypeData::UpdateFnInputMonotonic(UpdateFnInputMonotonicData {
                    input: p.input.to_string(),
                    target: p.target.to_string(),
                    value: p.value,
                    context: p.context.clone(),
                })
            }
            StatPropertyType::FnInputEssential(p) => {
                StatPropertyTypeData::FnInputEssential(FnInputEssentialData {
                    input_index: p.input_index,
                    target: p.target.to_string(),
                    value: p.value,
                    context: p.context.clone(),
                })
            }
            StatPropertyType::FnInputMonotonic(p) => {
                StatPropertyTypeData::FnInputMonotonic(FnInputMonotonicData {
                    input_index: p.input_index,
                    target: p.target.to_string(),
                    value: p.value,
                    context: p.context.clone(),
                })
            }
        };
        Self::new_raw(id.as_str(), name, variant)
    }

    /// Extract the corresponding `StatProperty` instance from this `StatPropertyData`.
    pub fn to_property(&self) -> Result<StatProperty, String> {
        let name = self.name.as_str();
        match &self.variant {
            StatPropertyTypeData::GenericStatProp(p) => StatProperty::mk_generic(name, &p.formula),
            StatPropertyTypeData::FnInputMonotonic(p) => StatProperty::mk_fn_input_monotonic(
                name,
                p.input_index,
                UninterpretedFnId::new(&p.target)?,
                p.value,
                p.context.clone(),
            ),
            StatPropertyTypeData::FnInputEssential(p) => StatProperty::mk_fn_input_essential(
                name,
                p.input_index,
                UninterpretedFnId::new(&p.target)?,
                p.value,
                p.context.clone(),
            ),
            StatPropertyTypeData::UpdateFnInputMonotonic(p) => {
                StatProperty::mk_update_fn_input_monotonic(
                    name,
                    VarId::new(&p.input)?,
                    VarId::new(&p.target)?,
                    p.value,
                    p.context.clone(),
                )
            }
            StatPropertyTypeData::UpdateFnInputEssential(p) => {
                StatProperty::mk_update_fn_input_essential(
                    name,
                    VarId::new(&p.input)?,
                    VarId::new(&p.target)?,
                    p.value,
                    p.context.clone(),
                )
            }
        }
    }

    /// **(internal)** Shorthand to create new `StatPropertyData` instance given all its fields.
    fn new_raw(id: &str, name: &str, variant: StatPropertyTypeData) -> StatPropertyData {
        StatPropertyData {
            id: id.to_string(),
            name: name.to_string(),
            variant,
        }
    }
}
