use crate::sketchbook::ids::{DynPropertyId, StatPropertyId};

/// **(internal)** Definition and methods for `PropertyManager`.
mod _manager;
/// Structs and utilities regarding dynamic properties.
pub mod dynamic_props;
/// Structs and utilities regarding static properties.
pub mod static_props;

pub use _manager::PropertyManager;
pub use dynamic_props::{DynProperty, HctlFormula};
pub use static_props::{FirstOrderFormula, StatProperty};

/// An iterator over all <`DynPropertyId`, `DynProperty`> pairs of a `PropertyManager`.
pub type DynPropIterator<'a> = std::collections::hash_map::Iter<'a, DynPropertyId, DynProperty>;

/// An iterator over all <`StatPropertyId`, `StatProperty`> pairs of a `PropertyManager`.
pub type StatPropIterator<'a> = std::collections::hash_map::Iter<'a, StatPropertyId, StatProperty>;
