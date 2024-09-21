/// Encode data and template properties into HCTL.
pub mod encode;
/// Evaluate all kinds of dynamic properties.
pub mod eval;
/// Prepare graph and symbolic context to handle all dynamic properties.
pub mod prepare_graph;
/// Wrappers for algorithms for trap space computation.
pub mod trap_spaces;

/// Enum of possible variants of data to encode.
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum DataEncodingType {
    Attractor,
    FixedPoint,
}
