#[macro_use]
extern crate lazy_static;
extern crate core;

/// Algorithms for the inference process.
pub mod algorithms;
/// State of the inference analysis tab.
pub mod analysis;
/// General functionality for window mechanism, event stack, tab management, and more.
pub mod app;
/// Logging utilities.
pub mod logging;
/// State of the sketchbook editor tab.
pub mod sketchbook;
