use serde::{Deserialize, Serialize};

/// Structs and utility methods that can be used for communication with frontend.
pub mod data_structs;
/// Definitions and utilities for type-safe identifiers of various components.
pub mod ids;
/// Structs and utility methods regarding the layout of the Regulations editor.
pub mod layout;
/// Structs and utility methods regarding the model of the Regulations editor.
pub mod model;
/// Structs and utility methods regarding observations and datasets.
pub mod observations;
/// Classes and utility methods regarding properties.
pub mod properties;

/// Utility functions specifically related to events.
mod event_utils;
/// General utilities used throughout the module (e.g., serialization helper methods).
pub mod utils;

/// **(internal)** Tests for the event-based API of various top-level components.
#[cfg(test)]
mod _tests_events;

/// Trait that implements `to_json_str` and `from_json_str` wrappers to serialize and
/// deserialize objects, utilizing [serde_json].
///
/// All of the structs implementing `JsonSerde` must implement traits `Serialize` and `Deserialize`.
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
