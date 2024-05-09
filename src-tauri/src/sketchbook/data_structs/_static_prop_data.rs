use crate::sketchbook::ids::{StatPropertyId, UninterpretedFnId, VarId};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::static_props::{SimpleStatPropertyType, StatPropertyType};
use crate::sketchbook::properties::StatProperty;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

fn input_id_to_index(input_id: &Option<String>) -> Result<Option<usize>, String> {
    match input_id {
        Some(s) if s.starts_with("var") && s[3..].chars().all(char::is_numeric) => s[3..]
            .parse::<usize>()
            .map(Some)
            .map_err(|e| format!("{:?}", e)),
        None => Ok(None),
        _ => Err("Input ID has invalid format, must be `varN`".to_string()),
    }
}

fn input_index_to_id(input_index: usize) -> String {
    format!("var{}", input_index)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericStatPropData {
    pub formula: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegulationEssentialData {
    pub input: Option<String>,
    pub target: Option<String>,
    pub value: Essentiality,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FnInputEssentialData {
    pub input: Option<String>,
    pub target: Option<String>,
    pub value: Essentiality,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegulationMonotonicData {
    pub input: Option<String>,
    pub target: Option<String>,
    pub value: Monotonicity,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FnInputMonotonicData {
    pub input: Option<String>,
    pub target: Option<String>,
    pub value: Monotonicity,
    pub context: Option<String>,
}

/// Structure for receiving data to create default dynamic properties. For this, only the ID
/// and simple variant are needed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatPropertyDefaultData {
    pub id: String,
    pub variant: SimpleStatPropertyType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "variant")]
pub enum StatPropertyTypeData {
    GenericStatProp(GenericStatPropData),
    RegulationEssential(RegulationEssentialData),
    RegulationEssentialContext(RegulationEssentialData),
    FnInputEssential(FnInputEssentialData),
    FnInputEssentialContext(FnInputEssentialData),
    RegulationMonotonic(RegulationMonotonicData),
    RegulationMonotonicContext(RegulationMonotonicData),
    FnInputMonotonic(FnInputMonotonicData),
    FnInputMonotonicContext(FnInputMonotonicData),
}

/// Structure for sending data about static properties to the frontend.
///
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe pub structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatPropertyData {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub variant: StatPropertyTypeData,
}

impl<'de> JsonSerde<'de> for StatPropertyData {}
impl<'de> JsonSerde<'de> for StatPropertyDefaultData {}

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
            StatPropertyType::RegulationEssential(p) => {
                StatPropertyTypeData::RegulationEssential(RegulationEssentialData {
                    input: p.input.as_ref().map(|i| i.to_string()),
                    target: p.target.as_ref().map(|i| i.to_string()),
                    value: p.value,
                    context: None,
                })
            }
            StatPropertyType::RegulationEssentialContext(p) => {
                StatPropertyTypeData::RegulationEssentialContext(RegulationEssentialData {
                    input: p.input.as_ref().map(|i| i.to_string()),
                    target: p.target.as_ref().map(|i| i.to_string()),
                    value: p.value,
                    context: p.context.clone(),
                })
            }
            StatPropertyType::RegulationMonotonic(p) => {
                StatPropertyTypeData::RegulationMonotonic(RegulationMonotonicData {
                    input: p.input.as_ref().map(|i| i.to_string()),
                    target: p.target.as_ref().map(|i| i.to_string()),
                    value: p.value,
                    context: None,
                })
            }
            StatPropertyType::RegulationMonotonicContext(p) => {
                StatPropertyTypeData::RegulationMonotonicContext(RegulationMonotonicData {
                    input: p.input.as_ref().map(|i| i.to_string()),
                    target: p.target.as_ref().map(|i| i.to_string()),
                    value: p.value,
                    context: p.context.clone(),
                })
            }
            StatPropertyType::FnInputEssential(p) => {
                StatPropertyTypeData::FnInputEssential(FnInputEssentialData {
                    input: p.input_index.map(input_index_to_id),
                    target: p.target.as_ref().map(|i| i.to_string()),
                    value: p.value,
                    context: None,
                })
            }
            StatPropertyType::FnInputEssentialContext(p) => {
                StatPropertyTypeData::FnInputEssentialContext(FnInputEssentialData {
                    input: p.input_index.map(input_index_to_id),
                    target: p.target.as_ref().map(|i| i.to_string()),
                    value: p.value,
                    context: p.context.clone(),
                })
            }
            StatPropertyType::FnInputMonotonic(p) => {
                StatPropertyTypeData::FnInputMonotonic(FnInputMonotonicData {
                    input: p.input_index.map(input_index_to_id),
                    target: p.target.as_ref().map(|i| i.to_string()),
                    value: p.value,
                    context: None,
                })
            }
            StatPropertyType::FnInputMonotonicContext(p) => {
                StatPropertyTypeData::FnInputMonotonicContext(FnInputMonotonicData {
                    input: p.input_index.map(input_index_to_id),
                    target: p.target.as_ref().map(|i| i.to_string()),
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
                input_id_to_index(&p.input)?,
                p.target
                    .as_ref()
                    .and_then(|t| UninterpretedFnId::new(t).ok()),
                p.value,
            ),
            StatPropertyTypeData::FnInputMonotonicContext(p) => {
                StatProperty::mk_fn_input_monotonic_context(
                    name,
                    input_id_to_index(&p.input)?,
                    p.target
                        .as_ref()
                        .and_then(|t| UninterpretedFnId::new(t).ok()),
                    p.value,
                    p.context.clone().ok_or("Context missing.")?,
                )
            }
            StatPropertyTypeData::FnInputEssential(p) => StatProperty::mk_fn_input_essential(
                name,
                input_id_to_index(&p.input)?,
                p.target
                    .as_ref()
                    .and_then(|t| UninterpretedFnId::new(t).ok()),
                p.value,
            ),
            StatPropertyTypeData::FnInputEssentialContext(p) => {
                StatProperty::mk_fn_input_essential_context(
                    name,
                    input_id_to_index(&p.input)?,
                    p.target
                        .as_ref()
                        .and_then(|t| UninterpretedFnId::new(t).ok()),
                    p.value,
                    p.context.clone().ok_or("Context missing.")?,
                )
            }
            StatPropertyTypeData::RegulationMonotonic(p) => StatProperty::mk_regulation_monotonic(
                name,
                p.input.as_ref().and_then(|i| VarId::new(i).ok()),
                p.target.as_ref().and_then(|t| VarId::new(t).ok()),
                p.value,
            ),
            StatPropertyTypeData::RegulationMonotonicContext(p) => {
                StatProperty::mk_regulation_monotonic_context(
                    name,
                    p.input.as_ref().and_then(|i| VarId::new(i).ok()),
                    p.target.as_ref().and_then(|t| VarId::new(t).ok()),
                    p.value,
                    p.context.clone().ok_or("Context missing.")?,
                )
            }
            StatPropertyTypeData::RegulationEssential(p) => StatProperty::mk_regulation_essential(
                name,
                p.input.as_ref().and_then(|i| VarId::new(i).ok()),
                p.target.as_ref().and_then(|t| VarId::new(t).ok()),
                p.value,
            ),
            StatPropertyTypeData::RegulationEssentialContext(p) => {
                StatProperty::mk_regulation_essential_context(
                    name,
                    p.input.as_ref().and_then(|i| VarId::new(i).ok()),
                    p.target.as_ref().and_then(|t| VarId::new(t).ok()),
                    p.value,
                    p.context.clone().ok_or("Context missing.")?,
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
