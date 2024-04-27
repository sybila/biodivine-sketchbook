use crate::sketchbook::ids::{DynPropertyId, StatPropertyId};
use crate::sketchbook::properties::{
    DynPropIterator, DynProperty, PropertyManager, StatPropIterator, StatProperty,
};
use crate::sketchbook::utils::assert_ids_unique;
use std::collections::HashMap;

impl PropertyManager {
    /// Instantiate `PropertyManager` with empty sets of properties.
    pub fn new_empty() -> PropertyManager {
        PropertyManager {
            dyn_properties: HashMap::new(),
            stat_properties: HashMap::new(),
        }
    }

    /// Instantiate `PropertyManager` with (generic) dynamic and static properties given as a list
    /// of ID-name-formula tuples.
    pub fn new_from_formulae(
        dyn_properties: Vec<(&str, &str, &str)>,
        stat_properties: Vec<(&str, &str, &str)>,
    ) -> Result<PropertyManager, String> {
        let mut manager = PropertyManager::new_empty();

        let dyn_prop_ids = dyn_properties.iter().map(|pair| pair.0).collect();
        assert_ids_unique(&dyn_prop_ids)?;

        let stat_prop_ids = stat_properties.iter().map(|pair| pair.0).collect();
        assert_ids_unique(&stat_prop_ids)?;

        for (id, name, formula) in dyn_properties {
            let prop_id = DynPropertyId::new(id)?;
            manager
                .dyn_properties
                .insert(prop_id, DynProperty::mk_generic(name, formula)?);
        }
        for (id, name, formula) in stat_properties {
            let prop_id = StatPropertyId::new(id)?;
            manager
                .stat_properties
                .insert(prop_id, StatProperty::mk_generic(name, formula)?);
        }
        Ok(manager)
    }

    /// Instantiate `PropertyManager` with dynamic and static properties given as a list
    /// of ID-property pairs.
    pub fn new_from_properties(
        dyn_properties: Vec<(&str, DynProperty)>,
        stat_properties: Vec<(&str, StatProperty)>,
    ) -> Result<PropertyManager, String> {
        let mut manager = PropertyManager::new_empty();

        let dyn_prop_ids = dyn_properties.iter().map(|pair| pair.0).collect();
        assert_ids_unique(&dyn_prop_ids)?;

        let stat_prop_ids = stat_properties.iter().map(|pair| pair.0).collect();
        assert_ids_unique(&stat_prop_ids)?;

        for (id, prop) in dyn_properties {
            manager.dyn_properties.insert(DynPropertyId::new(id)?, prop);
        }
        for (id, prop) in stat_properties {
            manager
                .stat_properties
                .insert(StatPropertyId::new(id)?, prop);
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
