/// Encode data and template properties into HCTL.
pub mod encode;
/// Evaluate all kinds of dynamic properties.
pub mod eval;
/// Prepare graph and symbolic context to handle all dynamic properties.
pub mod prepare_graph;

/// Internal algorithms for attractor computation (adapted from AEON).
mod _attractors;
/// Internal algorithms for trap space computation (adapted from lib-param-bn).
pub mod _trap_spaces;

/// Enum of possible variants of data to encode.
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum DataEncodingType {
    Attractor,
    FixedPoint,
}
