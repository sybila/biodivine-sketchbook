use crate::sketchbook::{Observability, Regulation, RegulationSign};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RegulationData {
    pub regulator: String,
    pub target: String,
    pub sign: RegulationSign,
    pub observable: Observability,
}

impl RegulationData {
    pub fn new(
        regulator: String,
        target: String,
        observable: Observability,
        sign: RegulationSign,
    ) -> RegulationData {
        RegulationData {
            regulator,
            target,
            observable,
            sign,
        }
    }

    pub fn new_from_reg(
        reg: &Regulation,
    ) -> RegulationData {
        RegulationData {
            regulator: reg.get_regulator().to_string(),
            target:  reg.get_regulator().to_string(),
            observable: reg.get_observability().clone(),
            sign: reg.get_sign().clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct VariableData {
    pub id: String,
    pub name: String,
}

impl VariableData {
    pub fn new(id: String, name: String) -> VariableData {
        VariableData {
            id,
            name
        }
    }
}
