use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

/// **(internal)** Utility functions specifically related to events.
mod event_utils;
/// **(internal)** General utilities used throughout the module (e.g., serialization
/// helper methods).
mod utils;

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

/// Trait implementing functionality relevant for all manager structs (such as `ModelState`,
/// `ObservationManager`, ...).
pub trait Manager {
    /// Generate an ID of type `T` for a certain component of a manager (e.g., generate a
    /// `VariableId` for a `Variable` in a `ModelState`).
    ///
    /// The id is generated based on provided `ideal_id`. In best case, it is used directly.
    /// If this would cause some collisions, it is modified until the ID is unique.
    ///
    /// Method `is_taken` is provided to check whether a generated id is already taken (non-unique),
    /// and `max_idx` specifies maximum number that the component might need appended to make it
    /// unique (e.g., for a variable, it would be total number of variables in use).
    fn generate_id<T>(
        &self,
        ideal_id: &str,
        is_taken: &dyn Fn(&Self, &T) -> bool,
        max_idx: usize,
    ) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        // first try to use the `ideal_id`
        if let Ok(id) = T::from_str(ideal_id) {
            // the id must not be valid in the network already (that would mean it is already used)
            if !is_taken(self, &id) {
                return id;
            }
        }

        // try to transform the id by removing invalid characters
        let mut transformed_id: String = ideal_id
            .chars()
            .filter(|ch| ch.is_alphanumeric() || *ch == '_')
            .collect();
        // and if the first character is not a letter, add prefix 'v_'
        if transformed_id.starts_with(|ch: char| !ch.is_alphabetic()) {
            transformed_id.insert_str(0, "v_");
        }

        if let Ok(id) = T::from_str(transformed_id.as_str()) {
            // the id must not be valid in the network already (that would mean it is already used)
            if !is_taken(self, &id) {
                return id;
            }
        }

        // finally, append a number at the end of id
        // start searching at 0, until we try `max_idx` options
        for n in 0..max_idx {
            let id = T::from_str(format!("{}_{}", transformed_id, n).as_str()).unwrap();
            if !is_taken(self, &id) {
                return id;
            }
        }

        // this must be valid, we already tried more than `max_idx` options
        T::from_str(format!("{}_{}", transformed_id, max_idx).as_str()).unwrap()
    }
}
