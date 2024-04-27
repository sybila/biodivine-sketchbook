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
pub struct UpdateFnInputEssential {
    pub input: VarId,
    pub target: VarId,
    pub value: Essentiality,
    pub context: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UpdateFnInputMonotonic {
    pub input: VarId,
    pub target: VarId,
    pub value: Monotonicity,
    pub context: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FnInputEssential {
    pub input_index: usize,
    pub target: UninterpretedFnId,
    pub value: Essentiality,
    pub context: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FnInputMonotonic {
    pub input_index: usize,
    pub target: UninterpretedFnId,
    pub value: Monotonicity,
    pub context: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum StatPropertyType {
    FnInputEssential(FnInputEssential),
    FnInputMonotonic(FnInputMonotonic),
    UpdateFnInputEssential(UpdateFnInputEssential),
    UpdateFnInputMonotonic(UpdateFnInputMonotonic),
    GenericStatProp(GenericStatProp),
}
