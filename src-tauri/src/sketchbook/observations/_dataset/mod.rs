use crate::sketchbook::ids::{ObservationId, VarId};
use crate::sketchbook::observations::{DataCategory, Observation};
use crate::sketchbook::JsonSerde;
use std::collections::HashMap;

/// **(internal)** Basic utility methods for `Dataset`.
mod _impl_dataset;
/// **(internal)** Implementation of partial event-based API to manipulate observations.
mod _impl_events;
/// **(internal)** Implementation of [Serialize] and [Deserialize] traits for `Dataset`.
mod _impl_serde;

/// An ordered list of observations (of potentially specified type) for given variables.
/// The order is important for some datasets, for example, to be able to capture time series.
///
/// `Dataset` provides classical Rust API for modifications. It also manages its observations
/// through event-based API. However, this API is limited, and only serves as an extension to that
/// of the `ObservationManager`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dataset {
    /// List of binarized observations.
    observations: Vec<Observation>,
    /// Variables captured by the observations.
    variables: Vec<VarId>,
    /// Category of this dataset.
    category: DataCategory,
    /// Index map from observation IDs to their index in vector, for faster searching.
    index_map: HashMap<ObservationId, usize>,
}

impl<'de> JsonSerde<'de> for Dataset {}
