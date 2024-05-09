use crate::sketchbook::ids::{DatasetId, DynPropertyId, ObservationId};
use crate::sketchbook::properties::dynamic_props::{DynPropertyType, SimpleDynPropertyType};
use crate::sketchbook::properties::DynProperty;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericDynPropData {
    pub formula: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExistsFixedPointData {
    pub dataset: Option<String>,
    pub observation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrapSpaceData {
    pub dataset: Option<String>,
    pub observation: Option<String>,
    pub minimal: bool,
    pub nonpercolable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrajectoryData {
    pub dataset: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttractorCountData {
    pub minimal: usize,
    pub maximal: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HasAttractorData {
    pub dataset: Option<String>,
    pub observation: Option<String>,
}

/// Structure for receiving data to create default dynamic properties. For this, only the ID
/// and simple variant are needed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynPropertyDefaultData {
    pub id: String,
    pub variant: SimpleDynPropertyType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "variant")]
pub enum DynPropertyTypeData {
    GenericDynProp(GenericDynPropData),
    ExistsFixedPoint(ExistsFixedPointData),
    ExistsTrapSpace(ExistsTrapSpaceData),
    ExistsTrajectory(ExistsTrajectoryData),
    AttractorCount(AttractorCountData),
    HasAttractor(HasAttractorData),
}

/// Structure for sending data about dynamic properties to the frontend.
///
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynPropertyData {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub variant: DynPropertyTypeData,
}

impl<'de> JsonSerde<'de> for DynPropertyData {}
impl<'de> JsonSerde<'de> for DynPropertyDefaultData {}

impl DynPropertyData {
    /// Shorthand to create new generic `DynPropertyData` instance given a properties
    /// `id`, `name`, and `formula`.
    pub fn new_generic(id: &str, name: &str, formula: &str) -> DynPropertyData {
        let variant = DynPropertyTypeData::GenericDynProp(GenericDynPropData {
            formula: formula.to_string(),
        });
        Self::new_raw(id, name, variant)
    }

    /// Create new `DynPropertyData` object given a reference to a property and its `id`.
    pub fn from_property(id: &DynPropertyId, property: &DynProperty) -> DynPropertyData {
        let name = property.get_name();
        let variant = match property.get_prop_data() {
            DynPropertyType::GenericDynProp(p) => {
                DynPropertyTypeData::GenericDynProp(GenericDynPropData {
                    formula: p.raw_formula.to_string(),
                })
            }
            DynPropertyType::ExistsFixedPoint(p) => {
                DynPropertyTypeData::ExistsFixedPoint(ExistsFixedPointData {
                    dataset: p.dataset.as_ref().map(|i| i.to_string()),
                    observation: p.observation.as_ref().map(|i| i.to_string()),
                })
            }
            DynPropertyType::ExistsTrapSpace(p) => {
                DynPropertyTypeData::ExistsTrapSpace(ExistsTrapSpaceData {
                    dataset: p.dataset.clone().map(|i| i.to_string()),
                    observation: p.observation.clone().map(|i| i.to_string()),
                    minimal: p.minimal,
                    nonpercolable: p.nonpercolable,
                })
            }
            DynPropertyType::ExistsTrajectory(p) => {
                DynPropertyTypeData::ExistsTrajectory(ExistsTrajectoryData {
                    dataset: p.dataset.as_ref().map(|i| i.to_string()),
                })
            }
            DynPropertyType::HasAttractor(p) => {
                DynPropertyTypeData::HasAttractor(HasAttractorData {
                    dataset: p.dataset.as_ref().map(|i| i.to_string()),
                    observation: p.observation.as_ref().map(|o| o.to_string()),
                })
            }
            DynPropertyType::AttractorCount(p) => {
                DynPropertyTypeData::AttractorCount(AttractorCountData {
                    minimal: p.minimal,
                    maximal: p.maximal,
                })
            }
        };
        Self::new_raw(id.as_str(), name, variant)
    }

    /// Extract the corresponding `DynProperty` instance from this `DynPropertyData`.
    pub fn to_property(&self) -> Result<DynProperty, String> {
        let name = self.name.as_str();
        match &self.variant {
            DynPropertyTypeData::GenericDynProp(p) => DynProperty::mk_generic(name, &p.formula),
            DynPropertyTypeData::ExistsFixedPoint(p) => DynProperty::mk_fixed_point(
                name,
                p.dataset.as_ref().and_then(|t| DatasetId::new(t).ok()),
                p.observation
                    .as_ref()
                    .and_then(|t| ObservationId::new(t).ok()),
            ),
            DynPropertyTypeData::ExistsTrapSpace(p) => DynProperty::mk_trap_space(
                name,
                p.dataset.as_ref().and_then(|t| DatasetId::new(t).ok()),
                p.observation
                    .as_ref()
                    .and_then(|t| ObservationId::new(t).ok()),
                p.minimal,
                p.nonpercolable,
            ),
            DynPropertyTypeData::ExistsTrajectory(p) => {
                let dataset = p.dataset.as_ref().and_then(|t| DatasetId::new(t).ok());
                DynProperty::mk_trajectory(name, dataset)
            }
            DynPropertyTypeData::HasAttractor(p) => DynProperty::mk_has_attractor(
                name,
                p.dataset.as_ref().and_then(|t| DatasetId::new(t).ok()),
                p.observation
                    .as_ref()
                    .and_then(|t| ObservationId::new(t).ok()),
            ),
            DynPropertyTypeData::AttractorCount(p) => {
                DynProperty::mk_attractor_count(name, p.minimal, p.maximal)
            }
        }
    }

    /// **(internal)** Shorthand to create new `DynPropertyData` instance given all its fields.
    fn new_raw(id: &str, name: &str, variant: DynPropertyTypeData) -> DynPropertyData {
        DynPropertyData {
            id: id.to_string(),
            name: name.to_string(),
            variant,
        }
    }
}
