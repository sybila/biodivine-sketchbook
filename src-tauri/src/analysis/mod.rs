/// Structures and utilities to track results of analysis.
pub mod analysis_results;
/// Structures and utilities to track the whole state of analysis.
pub mod analysis_state;
/// Structs and utility methods that can be used for communication with frontend.
pub mod data_structs;

/// Utilities to generate SymbolicContext and SymbolicAsyncGraph instances with enough extra
/// variables to be able to represent all HCTL and FOL variables.
mod context_utils;
/// Structures and methods to run the whole inference process.
/// This involves the general workflow, the details are in a separate module [analysis].
mod inference_solver;
/// Utilities to sample and download networks.
/// Some functionality is taken from our repository [biodivine-bn-classifier].
mod sampling_networks;
