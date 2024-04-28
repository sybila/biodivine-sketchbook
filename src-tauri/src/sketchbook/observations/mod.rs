use crate::sketchbook::ids::DatasetId;

/// **(internal)** Definition and methods for `DataCategory`.
mod _data_category;
/// **(internal)** Definition and methods for `Dataset`.
mod _dataset;
/// **(internal)** Definition and methods for `ObservationManager`.
mod _manager;
/// **(internal)** Definition and methods for `Observation`.
mod _observation;
/// **(internal)** Definition and methods for `VarValue`.
mod _var_value;

pub use _data_category::DataCategory;
pub use _dataset::Dataset;
pub use _manager::ObservationManager;
pub use _observation::Observation;
pub use _var_value::VarValue;

/// An iterator over all <`DatasetId`, `Dataset`> pairs of a `ObservationManager`.
pub type DatasetIterator<'a> = std::collections::hash_map::Iter<'a, DatasetId, Dataset>;
