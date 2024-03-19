use crate::sketchbook::properties::DynamicProperty;
use crate::sketchbook::PropertyId;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyManager {
    properties: HashMap<PropertyId, DynamicProperty>,
}

impl Default for PropertyManager {
    /// Default object with no datasets.
    fn default() -> PropertyManager {
        PropertyManager::new_empty()
    }
}

impl PropertyManager {
    /// Instantiate `PropertyManager` with empty list of properties.
    fn new_empty() -> PropertyManager {
        PropertyManager {
            properties: HashMap::new(),
        }
    }
}
