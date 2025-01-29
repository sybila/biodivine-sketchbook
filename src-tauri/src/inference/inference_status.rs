use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Status of the inference computation.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum InferenceStatus {
    /// Inference solver instance is created.
    Created,
    /// The inference computation is started.
    Started,
    /// Sketch input is processed (BN object created, ...).
    ProcessedInputs,
    /// Symbolic context and graph for static props is created.
    GeneratedContextStatic,
    /// Symbolic context and graph for dynamic props is created.
    GeneratedContextDynamic,
    /// Static property is evaluated (can happen multiple times).
    EvaluatedStatic(String),
    /// All static properties are evaluated.
    EvaluatedAllStatic,
    /// Static property is evaluated (can happen multiple times).
    EvaluatedDynamic(String),
    /// All dynamic properties are evaluated.
    EvaluatedAllDynamic,
    /// Detected that sketch is unsatisfiable (can happen at the end or during computation).
    DetectedUnsat,
    /// Computation is successfully finished.
    FinishedSuccessfully,
    /// Computation is finished but unsuccessful (cancellation or processing error).
    Error,
}

/// Report on status of the computation, together with few details and a timestamp.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceStatusReport {
    /// New status of the inference computation.
    pub status: InferenceStatus,
    /// Optional number of remaining candidates (not needed for some status updates,
    /// like when computation starts or when it finishes with an error).
    pub num_candidates: Option<String>,
    /// Computation time (from the start) as a number of milliseconds.
    pub comp_time: u128,
    /// Message to be shown at the frontend.
    pub message: String,
}

impl<'de> JsonSerde<'de> for InferenceStatusReport {}

impl InferenceStatusReport {
    /// Create new `InferenceStatusReport` given all the details.
    pub fn new(
        status: InferenceStatus,
        num_candidates: Option<String>,
        comp_time: u128,
        message: &str,
    ) -> InferenceStatusReport {
        InferenceStatusReport {
            status,
            num_candidates,
            comp_time,
            message: message.to_string(),
        }
    }
}
