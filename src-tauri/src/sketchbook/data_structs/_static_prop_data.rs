use crate::sketchbook::ids::{StatPropertyId, UninterpretedFnId, VarId};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::static_props;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};
use static_props::{StatProperty, StatPropertyType};

/// **(internal)** Convert function input from its string format `varN` into its index `N`.
fn input_id_to_index(input_id: &Option<String>) -> Result<Option<usize>, String> {
    match input_id {
        Some(s) if s.starts_with("var") && s[3..].chars().all(char::is_numeric) => s[3..]
            .parse::<usize>()
            .map(Some)
            .map_err(|e| format!("{e:?}")),
        None => Ok(None),
        _ => Err("Input ID has invalid format, must be `varN`".to_string()),
    }
}

/// **(internal)** Convert function input from its index `N` into corresponding string `varN`.
fn input_index_to_id(input_index: usize) -> String {
    format!("var{input_index}")
}

/// Simplified variant to carry data regarding [static_props::GenericStatProp] static property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericStatPropData {
    pub formula: String,
}

/// Simplified variant to carry data regarding [static_props::RegulationEssential] static property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegulationEssentialData {
    pub input: Option<String>,
    pub target: Option<String>,
    pub value: Essentiality,
    pub context: Option<String>,
}

/// Simplified variant to carry data regarding [static_props::FnInputEssential] static property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FnInputEssentialData {
    pub input: Option<String>,
    pub target: Option<String>,
    pub value: Essentiality,
    pub context: Option<String>,
}

/// Simplified variant to carry data regarding [static_props::RegulationMonotonic] static property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegulationMonotonicData {
    pub input: Option<String>,
    pub target: Option<String>,
    pub value: Monotonicity,
    pub context: Option<String>,
}

/// Simplified variant to carry data regarding [static_props::FnInputMonotonic] static property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FnInputMonotonicData {
    pub input: Option<String>,
    pub target: Option<String>,
    pub value: Monotonicity,
    pub context: Option<String>,
}

/// Enum covering all variants of static properties and their necessary data.
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
    pub annotation: String,
    #[serde(flatten)]
    pub variant: StatPropertyTypeData,
}

impl JsonSerde<'_> for StatPropertyData {}

impl StatPropertyData {
    /// Shorthand to create new generic `StatPropertyData` instance given a properties
    /// `id`, `name`, `formula`, and `annotation`.
    pub fn new_generic(id: &str, name: &str, formula: &str, annot: &str) -> StatPropertyData {
        let variant = StatPropertyTypeData::GenericStatProp(GenericStatPropData {
            formula: formula.to_string(),
        });
        Self::new_raw(id, name, variant, annot)
    }

    /// Create new `StatPropertyData` object given a reference to a property and its `id`.
    pub fn from_property(id: &StatPropertyId, property: &StatProperty) -> StatPropertyData {
        let name = property.get_name();
        let annot = property.get_annotation();
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
        Self::new_raw(id.as_str(), name, variant, annot)
    }

    /// Extract the corresponding `StatProperty` instance from this `StatPropertyData`.
    pub fn to_property(&self) -> Result<StatProperty, String> {
        let name = self.name.as_str();
        let annot = self.annotation.as_str();
        let property = match &self.variant {
            StatPropertyTypeData::GenericStatProp(p) => {
                StatProperty::try_mk_generic(name, &p.formula)?.with_annotation(annot)
            }
            StatPropertyTypeData::FnInputMonotonic(p) => {
                let input = input_id_to_index(&p.input)?;
                let target = p
                    .target
                    .as_ref()
                    .and_then(|t| UninterpretedFnId::new(t).ok());
                StatProperty::mk_fn_input_monotonic(name, input, target, p.value)
                    .with_annotation(annot)
            }
            StatPropertyTypeData::FnInputMonotonicContext(p) => {
                let input = input_id_to_index(&p.input)?;
                let target = p
                    .target
                    .as_ref()
                    .and_then(|t| UninterpretedFnId::new(t).ok());
                let context = p.context.clone().ok_or("Context missing.")?;
                StatProperty::mk_fn_input_monotonic_context(name, input, target, p.value, context)
                    .with_annotation(annot)
            }
            StatPropertyTypeData::FnInputEssential(p) => {
                let input = input_id_to_index(&p.input)?;
                let target = p
                    .target
                    .as_ref()
                    .and_then(|t| UninterpretedFnId::new(t).ok());
                StatProperty::mk_fn_input_essential(name, input, target, p.value)
                    .with_annotation(annot)
            }
            StatPropertyTypeData::FnInputEssentialContext(p) => {
                let input = input_id_to_index(&p.input)?;
                let target = p
                    .target
                    .as_ref()
                    .and_then(|t| UninterpretedFnId::new(t).ok());
                let context = p.context.clone().ok_or("Context missing.")?;
                StatProperty::mk_fn_input_essential_context(name, input, target, p.value, context)
                    .with_annotation(annot)
            }
            StatPropertyTypeData::RegulationMonotonic(p) => {
                let input = p.input.as_ref().and_then(|i| VarId::new(i).ok());
                let target = p.target.as_ref().and_then(|t| VarId::new(t).ok());
                StatProperty::mk_regulation_monotonic(name, input, target, p.value)
                    .with_annotation(annot)
            }
            StatPropertyTypeData::RegulationMonotonicContext(p) => {
                let input = p.input.as_ref().and_then(|i| VarId::new(i).ok());
                let target = p.target.as_ref().and_then(|t| VarId::new(t).ok());
                let context = p.context.clone().ok_or("Context missing.")?;
                StatProperty::mk_regulation_monotonic_context(name, input, target, p.value, context)
                    .with_annotation(annot)
            }
            StatPropertyTypeData::RegulationEssential(p) => {
                let input = p.input.as_ref().and_then(|i| VarId::new(i).ok());
                let target = p.target.as_ref().and_then(|t| VarId::new(t).ok());
                StatProperty::mk_regulation_essential(name, input, target, p.value)
                    .with_annotation(annot)
            }
            StatPropertyTypeData::RegulationEssentialContext(p) => {
                let input = p.input.as_ref().and_then(|i| VarId::new(i).ok());
                let target = p.target.as_ref().and_then(|t| VarId::new(t).ok());
                let context = p.context.clone().ok_or("Context missing.")?;
                StatProperty::mk_regulation_essential_context(name, input, target, p.value, context)
                    .with_annotation(annot)
            }
        };
        Ok(property)
    }

    /// **(internal)** Shorthand to create new `StatPropertyData` instance given all its fields.
    fn new_raw(
        id: &str,
        name: &str,
        variant: StatPropertyTypeData,
        annot: &str,
    ) -> StatPropertyData {
        StatPropertyData {
            id: id.to_string(),
            name: name.to_string(),
            variant,
            annotation: annot.to_string(),
        }
    }
}
