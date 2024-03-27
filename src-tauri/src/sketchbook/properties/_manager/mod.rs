use crate::sketchbook::ids::PropertyId;
use crate::sketchbook::properties::DynamicProperty;
use crate::sketchbook::JsonSerde;
use std::collections::HashMap;

/// **(internal)** Basic utility methods for `PropertyManager`.
mod _impl_manager;
/// **(internal)** Implementation of [Serialize] and [Deserialize] traits for `PropertyManager`.
mod _impl_serde;
/// **(internal)** Implementation of event-based API for the [SessionState] trait.
mod _impl_session_state;

/// Class to manage all properties.
///
/// `PropertyManager` can be managed through its classical Rust API, as well as
/// through the external events (as it implements the `SessionState` event).
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyManager {
    properties: HashMap<PropertyId, DynamicProperty>,
}

impl<'de> JsonSerde<'de> for PropertyManager {}

impl Default for PropertyManager {
    /// Default manager instance with no datasets.
    fn default() -> PropertyManager {
        PropertyManager::new_empty()
    }
}
