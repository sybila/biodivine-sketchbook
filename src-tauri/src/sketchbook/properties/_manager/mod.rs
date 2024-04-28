use crate::sketchbook::ids::{DynPropertyId, StatPropertyId};
use crate::sketchbook::properties::{DynProperty, StatProperty};
use crate::sketchbook::{JsonSerde, Manager};
use std::collections::HashMap;

/// **(internal)** Implementation of the safe identifier generating.
mod _impl_id_generating;
/// **(internal)** Basic utility methods for `PropertyManager`.
mod _impl_manager;
/// **(internal)** Implementation of [Serialize] and [Deserialize] traits for `PropertyManager`.
mod _impl_serde;
/// **(internal)** Implementation of event-based API for the [SessionState] trait.
mod _impl_session_state;

/// Class to manage all properties of the sketch.
///
/// `PropertyManager` can be managed through its classical Rust API, as well as
/// through the external events (as it implements the `SessionState` event).
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyManager {
    dyn_properties: HashMap<DynPropertyId, DynProperty>,
    stat_properties: HashMap<StatPropertyId, StatProperty>,
}

impl<'de> JsonSerde<'de> for PropertyManager {}
impl Manager for PropertyManager {}

impl Default for PropertyManager {
    /// Default manager instance with no datasets.
    fn default() -> PropertyManager {
        PropertyManager::new_empty()
    }
}
