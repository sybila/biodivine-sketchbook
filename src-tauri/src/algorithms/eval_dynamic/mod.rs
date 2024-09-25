/// Encode data and template properties into HCTL.
pub mod encode;
/// Evaluate all kinds of dynamic properties.
pub mod eval;
/// Prepare graph and symbolic context to handle all dynamic properties.
pub mod prepare_graph;
/// Processed variants of dynamic properties for evaluation.
pub mod processed_props;

/// Internal algorithms for attractor computation (adapted from AEON).
mod _attractors;
/// Internal algorithms for trap space computation (adapted from lib-param-bn).
pub mod _trap_spaces;
