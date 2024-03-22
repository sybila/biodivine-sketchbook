use crate::sketchbook::properties::DynamicProperty;
use crate::sketchbook::PropertyId;
use std::collections::{HashMap, HashSet};

/// Class to manage all properties.
///
/// `PropertyManager` can be managed through its classical Rust API, as well as
/// through the external events (as it implements the `SessionState` event).
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyManager {
    properties: HashMap<PropertyId, DynamicProperty>,
}

impl Default for PropertyManager {
    /// Default manager instance with no datasets.
    fn default() -> PropertyManager {
        PropertyManager::new_empty()
    }
}

impl PropertyManager {
    /// Instantiate `PropertyManager` with empty list of properties.
    pub fn new_empty() -> PropertyManager {
        PropertyManager {
            properties: HashMap::new(),
        }
    }

    /// Instantiate `PropertyManager` with given list of ID-formula pairs.
    pub fn new_from_properties(properties: Vec<(&str, &str)>) -> Result<PropertyManager, String> {
        let mut manager = PropertyManager::new_empty();

        let prop_id_set = properties.iter().map(|pair| pair.0).collect::<HashSet<_>>();
        if prop_id_set.len() != properties.len() {
            return Err(format!(
                "Properties {:?} contain duplicate IDs.",
                properties
            ));
        }

        for (id, formula) in properties {
            let prop_id = PropertyId::new(id)?;
            manager
                .properties
                .insert(prop_id, DynamicProperty::try_from_str(formula)?);
        }
        Ok(manager)
    }
}
