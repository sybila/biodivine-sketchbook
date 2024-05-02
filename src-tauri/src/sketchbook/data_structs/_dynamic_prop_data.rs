use crate::sketchbook::ids::DynPropertyId;
use crate::sketchbook::properties::dynamic_props::DynPropertyType;
use crate::sketchbook::properties::DynProperty;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericDynPropData {
    pub formula: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExistsFixedPointData {
    pub dataset: String,
    pub observation: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrapSpaceData {
    pub dataset: String,
    pub observation: String,
    pub minimal: bool,
    pub non_percolable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrajectoryData {
    pub dataset: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttractorCountData {
    pub minimal: usize,
    pub maximal: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HasAttractorData {
    pub dataset: String,
    pub observation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
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
    variant: DynPropertyTypeData,
}

impl<'de> JsonSerde<'de> for DynPropertyData {}

impl DynPropertyData {
    /// Create new generic `DynPropertyData` instance given a properties `id`, `name`, and `formula`.
    pub fn new_generic(id: &str, name: &str, formula: &str) -> DynPropertyData {
        DynPropertyData {
            id: id.to_string(),
            name: name.to_string(),
            variant: DynPropertyTypeData::GenericDynProp(GenericDynPropData {
                formula: formula.to_string(),
            }),
        }
    }

    /// Create new `DynPropertyData` object given a reference to a property and its `id`.
    /// TODO: placeholder, missing implementation for other types of properties
    pub fn from_property(id: &DynPropertyId, property: &DynProperty) -> DynPropertyData {
        let name = property.get_name();
        match property.get_prop_data() {
            DynPropertyType::GenericDynProp(p) => {
                Self::new_generic(id.as_str(), name, &p.raw_formula)
            }
            _ => todo!(),
        }
    }

    /// Extract the corresponding `DynProperty` instance from this `DynPropertyData`.
    /// TODO: placeholder, missing implementation for other types of properties
    pub fn to_property(&self) -> Result<DynProperty, String> {
        let name = self.name.as_str();
        match &self.variant {
            DynPropertyTypeData::GenericDynProp(p) => DynProperty::mk_generic(name, &p.formula),
            _ => todo!(),
        }
    }
}
