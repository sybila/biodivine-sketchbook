use crate::sketchbook::ids::{DatasetId, DynPropertyId, ObservationId};
use crate::sketchbook::properties::dynamic_props;
use crate::sketchbook::JsonSerde;
use dynamic_props::{DynProperty, DynPropertyType};
use serde::{Deserialize, Serialize};

/// Simplified variant to carry data regarding [dynamic_props::GenericDynProp] dynamic property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericDynPropData {
    pub formula: String,
}

/// Simplified variant to carry data regarding [dynamic_props::ExistsFixedPoint] dynamic property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExistsFixedPointData {
    pub dataset: Option<String>,
    pub observation: Option<String>,
}

/// Simplified variant to carry data regarding [dynamic_props::ExistsTrapSpace] dynamic property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrapSpaceData {
    pub dataset: Option<String>,
    pub observation: Option<String>,
    pub minimal: bool,
    pub nonpercolable: bool,
}

/// Simplified variant to carry data regarding [dynamic_props::ExistsTrajectory] dynamic property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrajectoryData {
    pub dataset: Option<String>,
}

/// Simplified variant to carry data regarding [dynamic_props::AttractorCount] dynamic property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttractorCountData {
    pub minimal: usize,
    pub maximal: usize,
}

/// Simplified variant to carry data regarding [dynamic_props::HasAttractor] dynamic property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HasAttractorData {
    pub dataset: Option<String>,
    pub observation: Option<String>,
}

/// Enum covering all variants of dynamic properties and their necessary data.
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
    pub annotation: String,
    #[serde(flatten)]
    pub variant: DynPropertyTypeData,
}

impl JsonSerde<'_> for DynPropertyData {}

impl DynPropertyData {
    /// Shorthand to create new generic `DynPropertyData` instance given a properties
    /// `id`, `name`, `formula`, and `annotation`.
    pub fn new_generic(id: &str, name: &str, formula: &str, annotation: &str) -> DynPropertyData {
        let variant = DynPropertyTypeData::GenericDynProp(GenericDynPropData {
            formula: formula.to_string(),
        });
        Self::new_raw(id, name, variant, annotation)
    }

    /// Create new `DynPropertyData` object given a reference to a property and its `id`.
    pub fn from_property(id: &DynPropertyId, property: &DynProperty) -> DynPropertyData {
        let name = property.get_name();
        let annot = property.get_annotation();
        let variant = match property.get_prop_data() {
            DynPropertyType::GenericDynProp(p) => {
                // only need to save the raw formula (the input written by the user)
                // the processed syntactic tree and wild-cards can be ignored (and reconstructed later)
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
        Self::new_raw(id.as_str(), name, variant, annot)
    }

    /// Extract the corresponding `DynProperty` instance from this `DynPropertyData`.
    pub fn to_property(&self) -> Result<DynProperty, String> {
        let name = self.name.as_str();
        let annot = self.annotation.as_str();
        let property = match &self.variant {
            DynPropertyTypeData::GenericDynProp(p) => {
                DynProperty::try_mk_generic(name, &p.formula)?.with_annotation(annot)
            }
            DynPropertyTypeData::ExistsFixedPoint(p) => {
                let dataset = p.dataset.as_ref().and_then(|t| DatasetId::new(t).ok());
                let obs = p
                    .observation
                    .as_ref()
                    .and_then(|t| ObservationId::new(t).ok());
                DynProperty::mk_fixed_point(name, dataset, obs).with_annotation(annot)
            }
            DynPropertyTypeData::ExistsTrapSpace(p) => {
                let dataset = p.dataset.as_ref().and_then(|t| DatasetId::new(t).ok());
                let obs = p
                    .observation
                    .as_ref()
                    .and_then(|t| ObservationId::new(t).ok());
                DynProperty::mk_trap_space(name, dataset, obs, p.minimal, p.nonpercolable)
                    .with_annotation(annot)
            }
            DynPropertyTypeData::ExistsTrajectory(p) => {
                let dataset = p.dataset.as_ref().and_then(|t| DatasetId::new(t).ok());
                DynProperty::mk_trajectory(name, dataset).with_annotation(annot)
            }
            DynPropertyTypeData::HasAttractor(p) => {
                let dataset = p.dataset.as_ref().and_then(|t| DatasetId::new(t).ok());
                let obs = p
                    .observation
                    .as_ref()
                    .and_then(|t| ObservationId::new(t).ok());
                DynProperty::mk_has_attractor(name, dataset, obs).with_annotation(annot)
            }
            DynPropertyTypeData::AttractorCount(p) => {
                DynProperty::try_mk_attractor_count(name, p.minimal, p.maximal)?
                    .with_annotation(annot)
            }
        };
        Ok(property)
    }

    /// **(internal)** Shorthand to create new `DynPropertyData` instance given all its fields.
    fn new_raw(
        id: &str,
        name: &str,
        variant: DynPropertyTypeData,
        annotation: &str,
    ) -> DynPropertyData {
        DynPropertyData {
            id: id.to_string(),
            name: name.to_string(),
            annotation: annotation.to_string(),
            variant,
        }
    }
}
