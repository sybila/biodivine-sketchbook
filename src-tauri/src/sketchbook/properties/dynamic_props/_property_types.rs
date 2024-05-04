use crate::sketchbook::ids::{DatasetId, ObservationId};
use crate::sketchbook::properties::HctlFormula;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GenericDynProp {
    pub raw_formula: String,
    pub processed_formula: HctlFormula,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExistsFixedPoint {
    pub dataset: Option<DatasetId>,
    pub observation: Option<ObservationId>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrapSpace {
    pub dataset: Option<DatasetId>,
    pub observation: Option<ObservationId>,
    pub minimal: bool,
    pub nonpercolable: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrajectory {
    pub dataset: Option<DatasetId>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AttractorCount {
    pub minimal: usize,
    pub maximal: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct HasAttractor {
    pub dataset: Option<DatasetId>,
    pub observation: Option<ObservationId>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum DynPropertyType {
    GenericDynProp(GenericDynProp),
    ExistsFixedPoint(ExistsFixedPoint),
    ExistsTrapSpace(ExistsTrapSpace),
    ExistsTrajectory(ExistsTrajectory),
    AttractorCount(AttractorCount),
    HasAttractor(HasAttractor),
}
