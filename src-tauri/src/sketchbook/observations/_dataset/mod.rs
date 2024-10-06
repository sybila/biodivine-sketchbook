use crate::sketchbook::ids::{ObservationId, VarId};
use crate::sketchbook::observations::Observation;
use crate::sketchbook::Manager;
use std::collections::HashMap;

/// **(internal)** Basic utility methods for `Dataset`.
mod _impl_dataset;
/// **(internal)** Implementation of partial event-based API to manipulate observations.
mod _impl_events;
/// **(internal)** Implementation of the safe identifier generating.
mod _impl_id_generating;

/// An ordered list of observations for given variables.
/// The order is important for some datasets, for example, to be able to capture time series.
///
/// `Dataset` provides classical Rust API for modifications. It also manages its observations
/// through event-based API. However, this API is limited, and only serves as an extension to that
/// of the `ObservationManager`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dataset {
    name: String,
    /// List of binarized observations.
    observations: Vec<Observation>,
    /// Variables captured by the observations.
    variables: Vec<VarId>,
    /// Index map from observation IDs to their index in vector, for faster searching.
    index_map: HashMap<ObservationId, usize>,
}

// We give `Manager` trait to Dataset as it simplifies many things.
// It really behaves like a manager class, but is slightly different than the other ones.
impl Manager for Dataset {}
