/// **(internal)** Definition and methods for `DynProperty`.
mod _dynamic_property;
/// **(internal)** Definition and methods for `PropertyManager`.
mod _manager;
/// **(internal)** Definition and methods for `StatProperty`.
mod _static_property;

/// **(internal)** Utility functions for automatically generating HCTL formulae.
#[allow(dead_code)]
mod _mk_hctl_formulas;

use crate::sketchbook::ids::{DynPropertyId, StatPropertyId};
pub use _dynamic_property::DynProperty;
pub use _manager::PropertyManager;
pub use _static_property::StatProperty;

/// An iterator over all <`DynPropertyId`, `DynProperty`> pairs of a `PropertyManager`.
pub type DynPropIterator<'a> = std::collections::hash_map::Iter<'a, DynPropertyId, DynProperty>;

/// An iterator over all <`StatPropertyId`, `StatProperty`> pairs of a `PropertyManager`.
pub type StatPropIterator<'a> = std::collections::hash_map::Iter<'a, StatPropertyId, StatProperty>;
