use crate::generate_property_enums;
use crate::sketchbook::ids::{DatasetId, ObservationId};
use crate::sketchbook::properties::HctlFormula;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};
use std::mem::discriminant;

use super::WildCardProposition;

/// Variant of `DynProperty` requiring that a particular HCTL formula to be satisfied.
/// The struct carries the user-provided `raw_formula` string, as well as its processed
/// internal version `processed_formula`. The two may differ a bit.
///
/// In addition, the struct contains a list of `wild_cards`, special propositions
/// that are used in the formula that can represent various "high-level templates".
/// They have to be processed separately and the format may differ between the
/// original and processed formula.
///
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GenericDynProp {
    pub raw_formula: String,
    pub wild_cards: Vec<WildCardProposition>,
    pub processed_formula: HctlFormula,
}

/// Variant of `DynProperty` requiring existence of a fixed point corresponding to
/// a particular `observation` of a particular `dataset`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExistsFixedPoint {
    pub dataset: Option<DatasetId>,
    pub observation: Option<ObservationId>,
}

/// Variant of `DynProperty` requiring existence of a trap space corresponding to
/// a particular `observation` of a particular `dataset`.
/// Optionally, the required trap space might be required to be `minimal` or `non-percolable`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrapSpace {
    pub dataset: Option<DatasetId>,
    pub observation: Option<ObservationId>,
    pub minimal: bool,
    pub nonpercolable: bool,
}

/// Variant of `DynProperty` requiring existence of a trajectory between observations
/// of a particular `dataset` (in a given order).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExistsTrajectory {
    pub dataset: Option<DatasetId>,
}

/// Variant of `DynProperty` requiring that the number of attractors falls into the range
/// <minimal, maximal>.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AttractorCount {
    pub minimal: usize,
    pub maximal: usize,
}

/// Variant of `DynProperty` requiring one of the following (depending on whether `observation`
/// is specified or not):
/// 1) all observations of a particular dataset correspond to an attractor
/// 2) a particular `observation` of a particular `dataset` correspond to an attractor
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct HasAttractor {
    pub dataset: Option<DatasetId>,
    pub observation: Option<ObservationId>,
}

// Two versions of the enum to cover all variants of the dynamic properties.
// One contains the property data inside, the other one only the discriminants.
generate_property_enums!(
    /// Enum covering all variants of dynamic properties and their necessary data.
    DynPropertyType,
    /// Enum covering all variants of dynamic properties (only discriminants, no data).
    SimpleDynPropertyType, {
        GenericDynProp(GenericDynProp),
        ExistsFixedPoint(ExistsFixedPoint),
        ExistsTrapSpace(ExistsTrapSpace),
        ExistsTrajectory(ExistsTrajectory),
        AttractorCount(AttractorCount),
        HasAttractor(HasAttractor)
    }
);

impl JsonSerde<'_> for SimpleDynPropertyType {}

/// Check if two DynPropertyType instances are of the same variant.
pub fn are_same_dyn_variant(a: &DynPropertyType, b: &DynPropertyType) -> bool {
    discriminant(a) == discriminant(b)
}
