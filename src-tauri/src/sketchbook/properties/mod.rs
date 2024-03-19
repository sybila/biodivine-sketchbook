/// **(internal)** Definition and methods for `DynamicProperty`.
mod _dynamic_property;
/// **(internal)** Definition and methods for `PropertyManager`.
mod _property_manager;

/// **(internal)** Utility functions for automatically generating HCTL formulae.
#[allow(dead_code)]
mod _mk_formulas;

pub use _dynamic_property::DynamicProperty;
pub use _property_manager::PropertyManager;
