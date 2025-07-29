use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;
use utils::assert_ids_unique;

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
#[macro_use]
pub mod properties;
/// Utilities regarding conversion of BN components and similar.
pub mod bn_utils;

/// **(internal)** Utility functions specifically related to events.
pub(crate) mod event_utils;

/// The main `Sketch` manager object and its utilities.
mod _sketch;
/// **(internal)** General utilities used throughout the module (e.g., serialization
/// helper methods).
mod utils;

/// **(internal)** Tests for the event-based API of various top-level components.
#[cfg(test)]
mod _tests_events;

pub use crate::sketchbook::_sketch::Sketch;

/// Trait that implements `to_json_str` and `from_json_str` wrappers to serialize and
/// deserialize objects, utilizing [serde_json].
///
/// All of the structs implementing `JsonSerde` must implement traits `Serialize` and `Deserialize`.
pub trait JsonSerde<'de>: Sized + Serialize + Deserialize<'de> {
    /// Wrapper for json serialization.
    fn to_json_str(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Wrapper for *pretty* json serialization with indentation.
    fn to_pretty_json_str(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
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
    /// If this would cause some collisions, it is modified by adding index at the end until
    /// the ID becomes unique.
    /// By specifying `start_index`, the index search starts directly at that number (e.g., when
    /// ideal ID is "var" and start index is 3, search for ID starts with "var_3", "var_4", ...)
    ///
    /// Method `is_taken` is provided to check whether a generated id is already taken (non-unique),
    /// and `num_indices` specifies maximum number of different indices that might be needed to be explored
    /// to make the id unique (e.g., for a variable, it would be total number of variables used).
    fn generate_id<T>(
        &self,
        ideal_id: &str,
        is_taken: &dyn Fn(&Self, &T) -> bool,
        num_indices: usize,
        start_index: Option<usize>,
    ) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        // first try to use the `ideal_id`
        if let Ok(id) = T::from_str(ideal_id) {
            // the id must not be used in the model already
            // also, if start index is given, this can not be used
            if !is_taken(self, &id) && start_index.is_none() {
                return id;
            }
        }

        // try to transform the id by removing invalid characters
        let mut transformed_id: String = ideal_id
            .chars()
            .filter(|ch| ch.is_alphanumeric() || *ch == '_')
            .collect();
        // and if the first character is not a letter, add prefix 'x_'
        if transformed_id.starts_with(|ch: char| !ch.is_alphabetic()) {
            transformed_id.insert_str(0, "x_");
        }

        if let Ok(id) = T::from_str(transformed_id.as_str()) {
            // the id must not be valid in the network already (that would mean it is already used)
            if !is_taken(self, &id) && start_index.is_none() {
                return id;
            }
        }

        // finally, try searching for a valid number to append at the end of the id
        // start searching at index 1 (or start-index), until we try `max_idx` options
        let start_index = start_index.unwrap_or(1);
        let last_index = start_index + num_indices;
        for n in start_index..last_index {
            let id = T::from_str(format!("{transformed_id}_{n}").as_str()).unwrap();
            if !is_taken(self, &id) {
                return id;
            }
        }

        // this must be valid, we already tried more than `max_idx` options
        T::from_str(format!("{transformed_id}_{last_index}").as_str()).unwrap()
    }

    /// Check that the list of (typesafe or string) IDs contains only unique IDs (no duplicates),
    /// and check that all of the IDs are already managed by the manager instance (this is
    /// important, for instance, when we need to change already existing elements).
    ///
    /// Manager class' method to assert ID validity must be provided.
    fn assert_ids_unique_and_used<T>(
        &self,
        id_list: &Vec<&str>,
        assert_id_is_managed: &dyn Fn(&Self, &T) -> Result<(), String>,
    ) -> Result<(), String>
    where
        T: Eq + Hash + Debug + FromStr,
        <T as FromStr>::Err: Debug,
    {
        assert_ids_unique(id_list)?;
        for &id_str in id_list {
            let id = T::from_str(id_str).map_err(|e| format!("{e:?}"))?;
            assert_id_is_managed(self, &id)?;
        }
        Ok(())
    }

    /// Check that the list of (typesafe or string) IDs contains only unique IDs (no duplicates),
    /// and check that all of the IDs are NOT yet managed by the manager instance, i.e.,
    /// they are fresh new values (this is important, for instance, when we need to add several new
    /// elements).
    ///
    /// Specific manager class' method to assert ID validity must be provided as `assert_id_is_new`.
    fn assert_ids_unique_and_new<T>(
        &self,
        id_list: &Vec<&str>,
        assert_id_is_new: &dyn Fn(&Self, &T) -> Result<(), String>,
    ) -> Result<(), String>
    where
        T: Eq + Hash + Debug + FromStr,
        <T as FromStr>::Err: Debug,
    {
        assert_ids_unique(id_list)?;
        for &id_str in id_list {
            let id = T::from_str(id_str).map_err(|e| format!("{e:?}"))?;
            assert_id_is_new(self, &id)?;
        }
        Ok(())
    }
}
