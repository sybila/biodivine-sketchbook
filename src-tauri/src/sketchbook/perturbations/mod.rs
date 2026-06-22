use crate::sketchbook::ids::PerturbationId;

/// **(internal)** Definition and methods for `PerturbationManager`.
mod _manager;

mod perturbation;

pub use _manager::PerturbationManager;
pub use perturbation::Perturbation;

/// An iterator over all <`PerturbationId`, `Perturbation`> pairs of a `PerturbationManager`.
pub type PerturbationIterator<'a> =
    std::collections::hash_map::Iter<'a, PerturbationId, Perturbation>;
