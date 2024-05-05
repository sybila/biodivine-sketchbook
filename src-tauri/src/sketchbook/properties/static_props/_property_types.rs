use crate::sketchbook::ids::{UninterpretedFnId, VarId};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::FirstOrderFormula;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GenericStatProp {
    pub raw_formula: String,
    pub processed_formula: FirstOrderFormula,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RegulationEssential {
    pub input: Option<VarId>,
    pub target: Option<VarId>,
    pub value: Essentiality,
    pub context: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RegulationMonotonic {
    pub input: Option<VarId>,
    pub target: Option<VarId>,
    pub value: Monotonicity,
    pub context: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FnInputEssential {
    pub input_index: Option<usize>,
    pub target: Option<UninterpretedFnId>,
    pub value: Essentiality,
    pub context: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FnInputMonotonic {
    pub input_index: Option<usize>,
    pub target: Option<UninterpretedFnId>,
    pub value: Monotonicity,
    pub context: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum StatPropertyType {
    FnInputEssential(FnInputEssential),
    FnInputMonotonic(FnInputMonotonic),
    FnInputEssentialContext(FnInputEssential),
    FnInputMonotonicContext(FnInputMonotonic),
    RegulationEssential(RegulationEssential),
    RegulationMonotonic(RegulationMonotonic),
    RegulationEssentialContext(RegulationEssential),
    RegulationMonotonicContext(RegulationMonotonic),
    GenericStatProp(GenericStatProp),
}
