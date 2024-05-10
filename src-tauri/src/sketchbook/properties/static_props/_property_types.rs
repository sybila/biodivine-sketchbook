use crate::generate_property_enums;
use crate::sketchbook::ids::{UninterpretedFnId, VarId};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::FirstOrderFormula;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};
use std::mem::discriminant;

/// Variant of `StatProperty` requiring that a particular first-order formula is satisfied.
/// The struct carries the user-provided `raw_formula` string, as well as its processed
/// internal version `processed_formula`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GenericStatProp {
    pub raw_formula: String,
    pub processed_formula: FirstOrderFormula,
}

/// Variant of `StatProperty` requiring that a regulation `input` -> `target` is essential,
/// either generally, or within optionally specified `context`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RegulationEssential {
    pub input: Option<VarId>,
    pub target: Option<VarId>,
    pub value: Essentiality,
    pub context: Option<String>,
}

/// Variant of `StatProperty` requiring that a regulation `input` -> `target` is monotonic,
/// either generally, or within optionally specified `context`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RegulationMonotonic {
    pub input: Option<VarId>,
    pub target: Option<VarId>,
    pub value: Monotonicity,
    pub context: Option<String>,
}

/// Variant of `StatProperty` requiring that an input (on specified `input_index`) of a
/// `target` function is essential - either generally, or within optionally specified `context`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FnInputEssential {
    pub input_index: Option<usize>,
    pub target: Option<UninterpretedFnId>,
    pub value: Essentiality,
    pub context: Option<String>,
}

/// Variant of `StatProperty` requiring that an input (on specified `input_index`) of a
/// `target` function is monotonic - either generally, or within optionally specified `context`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FnInputMonotonic {
    pub input_index: Option<usize>,
    pub target: Option<UninterpretedFnId>,
    pub value: Monotonicity,
    pub context: Option<String>,
}

// Two versions of the enum to cover all variants of the dynamic properties.
// One contains the property data inside, the other one only the discriminants.
generate_property_enums!(
    /// Enum covering all variants of static properties and their necessary data.
    StatPropertyType,
    /// Enum covering all variants of static properties (only discriminants, no data).
    SimpleStatPropertyType, {
        FnInputEssential(FnInputEssential),
        FnInputMonotonic(FnInputMonotonic),
        FnInputEssentialContext(FnInputEssential),
        FnInputMonotonicContext(FnInputMonotonic),
        RegulationEssential(RegulationEssential),
        RegulationMonotonic(RegulationMonotonic),
        RegulationEssentialContext(RegulationEssential),
        RegulationMonotonicContext(RegulationMonotonic),
        GenericStatProp(GenericStatProp)
    }
);

impl<'de> JsonSerde<'de> for SimpleStatPropertyType {}

/// Check if two StatPropertyType instances are of the same variant.
pub fn are_same_stat_variant(a: &StatPropertyType, b: &StatPropertyType) -> bool {
    discriminant(a) == discriminant(b)
}
