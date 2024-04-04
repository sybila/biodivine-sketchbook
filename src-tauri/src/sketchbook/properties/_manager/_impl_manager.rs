use crate::sketchbook::ids::PropertyId;
use crate::sketchbook::properties::{DynamicProperty, PropertyManager};
use std::collections::{HashMap, HashSet};

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

/// Observing the `PropertyManager`.
impl PropertyManager {
    /// The number of properties in this `PropertyManager`.
    pub fn num_properties(&self) -> usize {
        self.properties.len()
    }

    /// Check if there is a property with given Id.
    pub fn is_valid_property_id(&self, id: &PropertyId) -> bool {
        self.properties.contains_key(id)
    }
}
