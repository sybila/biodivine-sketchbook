use serde::{Deserialize, Serialize};

/// Utility functions specifically related to events.
mod event_utils;

/// Classes and utility methods that can be used for sending simplified data to frontend.
/// This includes simplified "data carriers" for variables, regulations, and layouts.
pub mod data_structs;
/// **(internal)** Classes and utility methods regarding the type-safe identifiers for
/// various components.
pub mod ids;
/// Classes and utility methods regarding the layout of the Regulations editor.
pub mod layout;
/// Classes and utility methods regarding the model of the Regulations editor.
pub mod model;
/// Classes and utility methods regarding the observations.
pub mod observations;
/// Classes and utility methods regarding the properties.
pub mod properties;
/// Utility functions used throughout the module.
pub mod utils;

/// **(internal)** Tests for the event-based API of various components.
#[cfg(test)]
mod _tests_events;

/// Trait that implements `to_json_str` and `from_json_str` wrappers, utilizing [serde_json].
/// All of the structs implementing `JsonSerde` must implement traits `Serialize` and
pub trait JsonSerde<'de>: Sized + Serialize + Deserialize<'de> {
    /// Wrapper for json serialization.
    fn to_json_str(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Wrapper for json de-serialization.
    fn from_json_str(s: &'de str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
