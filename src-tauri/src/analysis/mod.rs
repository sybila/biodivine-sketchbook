/// Structures and utilities to track results of analysis.
pub mod analysis_results;
/// Structures and utilities to track the whole state of analysis.
pub mod analysis_state;
/// Enum with various supported analysis types.
pub mod analysis_type;
/// Structs and utility methods that can be used for communication with frontend.
pub mod data_structs;

/// Structures and methods to run the whole inference process.
/// This involves the general workflow, the details are in a separate module [analysis].
mod inference_solver;
/// Utilities to sample and download networks.
/// Some functionality is taken from our repository [biodivine-bn-classifier].
mod sampling_networks;
