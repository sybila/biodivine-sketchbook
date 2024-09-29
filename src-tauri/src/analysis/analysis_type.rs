use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnalysisType {
    FullInference,
    StaticInference,
    DynamicInference,
}
