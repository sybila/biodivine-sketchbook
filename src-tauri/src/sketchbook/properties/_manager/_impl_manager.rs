use crate::sketchbook::ids::{DynPropertyId, StatPropertyId};
use crate::sketchbook::properties::{
    DynPropIterator, DynProperty, PropertyManager, StatPropIterator,
};
use std::collections::{HashMap, HashSet};

impl PropertyManager {
    /// Instantiate `PropertyManager` with empty sets of properties.
    pub fn new_empty() -> PropertyManager {
        PropertyManager {
            dyn_properties: HashMap::new(),
            stat_properties: HashMap::new(),
        }
    }

    /// Instantiate `PropertyManager` with dynamic properties given as a list of ID-formula pairs.
    pub fn new_from_dyn_properties(
        properties: Vec<(&str, &str)>,
    ) -> Result<PropertyManager, String> {
        let mut manager = PropertyManager::new_empty();

        let prop_id_set = properties.iter().map(|pair| pair.0).collect::<HashSet<_>>();
        if prop_id_set.len() != properties.len() {
            return Err(format!(
                "Properties {:?} contain duplicate IDs.",
                properties
            ));
        }

        for (id, formula) in properties {
            let prop_id = DynPropertyId::new(id)?;
            manager
                .dyn_properties
                .insert(prop_id, DynProperty::try_from_str(formula)?);
        }
        Ok(manager)
    }
}

/// Observing the `PropertyManager`.
impl PropertyManager {
    /// The number of dynamic properties in this `PropertyManager`.
    pub fn num_dyn_properties(&self) -> usize {
        self.dyn_properties.len()
    }

    /// The number of static properties in this `PropertyManager`.
    pub fn num_stat_properties(&self) -> usize {
        self.stat_properties.len()
    }

    /// Check if there is a dynamic property with given Id.
    pub fn is_valid_dyn_property_id(&self, id: &DynPropertyId) -> bool {
        self.dyn_properties.contains_key(id)
    }

    /// Check if there is a static property with given Id.
    pub fn is_valid_stat_property_id(&self, id: &StatPropertyId) -> bool {
        self.stat_properties.contains_key(id)
    }

    /// Return an iterator over all dynamic properties of this model.
    pub fn dyn_props(&self) -> DynPropIterator {
        self.dyn_properties.iter()
    }

    /// Return an iterator over all dynamic properties of this model.
    pub fn stat_props(&self) -> StatPropIterator {
        self.stat_properties.iter()
    }
}
