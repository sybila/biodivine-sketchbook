/// **(internal)** Utility methods for `DynamicProperty`.
mod _dynamic_property;

/// **(internal)** Utility functions for automatically generating HCTL formulae.
#[allow(dead_code)]
mod _mk_formulas;

pub use _dynamic_property::DynamicProperty;
