/// Structures and utilities to track final results of inference.
pub mod inference_results;
/// Structures and methods to run the whole inference process.
/// This module covers the general async workflow, the details regarding the actual
/// algorithms are in the module [crate::algorithms].
pub mod inference_solver;
/// Structures and utilities to track the whole state of inference.
pub mod inference_state;
/// Structures and utilities to track progress status of inference.
pub mod inference_status;
/// Enum with various supported inference types.
pub mod inference_type;
/// Struct with details regarding candidate sampling.
pub mod sampling_data;

/// Utilities to sample and download networks.
/// Some functionality is taken from our repository [biodivine-bn-classifier].
mod candidate_sampling;
/// Utilities to download results.
mod results_export;
/// Utilities to explore canditate update functions.
mod update_fn_details;

/// **(internal)** Several test scenarios for the inference procedure.
#[cfg(test)]
mod _test_inference;
