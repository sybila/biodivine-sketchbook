/// Structures and utilities to track the whole state of analysis.
pub mod analysis_state;
/// Structures and utilities to track final results of inference.
pub mod inference_results;
/// Structures and utilities to track progress status of inference.
pub mod inference_status;
/// Enum with various supported analysis types.
pub mod inference_type;
/// Struct with details regarding candidate sampling.
pub mod sampling_data;

/// Utilities to sample and download networks.
/// Some functionality is taken from our repository [biodivine-bn-classifier].
mod candidate_sampling;
/// Structures and methods to run the whole inference process.
/// This involves the general workflow, the details are in a separate module [analysis].
mod inference_solver;
/// Utilities to download results.
mod results_export;
/// Utilities to explore canditate update functions.
mod update_fn_details;

/// **(internal)** Several test scenarios for the inference procedure.
#[cfg(test)]
mod _test_inference;
