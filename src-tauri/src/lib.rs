#[macro_use]
extern crate lazy_static;
extern crate core;

/// All the internal algorithms for the inference process and low-level computation details.
pub mod algorithms;
/// General functionality for session management, windows, event stack, tab management, and more.
pub mod app;
/// State of the inference analysis and computation solvers.
pub mod inference;
/// Custom logging utilities.
pub mod logging;
/// State of the BN sketch, and management for most of the editor tab.
pub mod sketchbook;
