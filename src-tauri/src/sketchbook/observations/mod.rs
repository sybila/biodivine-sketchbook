/// **(internal)** Definition and methods for `Dataset`.
mod _dataset;
/// **(internal)** Definition and methods for `Observation`.
mod _observation;
/// **(internal)** Definition and methods for `ObservationType`.
mod _observation_manager;
/// **(internal)** Definition and methods for `ObservationType`.
mod _observation_type;
/// **(internal)** Definition and methods for `VarValue`.
mod _var_value;

use crate::sketchbook::DatasetId;
pub use _dataset::Dataset;
pub use _observation::Observation;
pub use _observation_manager::ObservationManager;
pub use _observation_type::ObservationType;
pub use _var_value::VarValue;

/// An iterator over all <`DatasetId`, `Dataset`> pairs of a `ObservationManager`.
pub type DatasetIterator<'a> = std::collections::hash_map::Iter<'a, DatasetId, Dataset>;
