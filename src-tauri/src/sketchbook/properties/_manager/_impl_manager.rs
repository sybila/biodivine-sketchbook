use crate::sketchbook::ids::{
    DatasetId, DynPropertyId, ObservationId, StatPropertyId, UninterpretedFnId, VarId,
};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::{
    DynPropIterator, DynProperty, PropertyManager, StatPropIterator, StatProperty,
};
use crate::sketchbook::utils::assert_ids_unique;
use std::collections::HashMap;
use std::str::FromStr;

/// Creating new instances of `PropertyManager`.
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

/// Editing `PropertyManager`.
impl PropertyManager {
    /// Add pre-generated dynamic property.
    pub fn add_raw_dynamic(&mut self, id: DynPropertyId, prop: DynProperty) -> Result<(), String> {
        self.assert_no_dynamic(&id)?;
        self.dyn_properties.insert(id, prop);
        Ok(())
    }

    /// Add pre-generated static property.
    pub fn add_raw_static(&mut self, id: StatPropertyId, prop: StatProperty) -> Result<(), String> {
        self.assert_no_static(&id)?;
        self.stat_properties.insert(id, prop);
        Ok(())
    }

    /// Add a new "generic" `DynProperty` instance with a given formula, which must be in a
    /// correct format (which is verified).
    pub fn add_dyn_generic(
        &mut self,
        id: DynPropertyId,
        name: &str,
        raw_formula: &str,
    ) -> Result<(), String> {
        self.assert_no_dynamic(&id)?;
        let property = DynProperty::mk_generic(name, raw_formula)?;
        self.dyn_properties.insert(id, property);
        Ok(())
    }

    /// Add a new `DynProperty` instance describing existence of a fixed point corresponding to
    /// a given observation.
    pub fn add_dyn_fixed_point(
        &mut self,
        id: DynPropertyId,
        name: &str,
        dataset: DatasetId,
        observation: ObservationId,
    ) -> Result<(), String> {
        self.assert_no_dynamic(&id)?;
        let property = DynProperty::mk_fixed_point(name, dataset, observation)?;
        self.dyn_properties.insert(id, property);
        Ok(())
    }

    /// Add a new `DynProperty` instance describing existence of a trap space corresponding to
    /// a given observation.
    pub fn add_dyn_trap_space(
        &mut self,
        id: DynPropertyId,
        name: &str,
        dataset: DatasetId,
        observation: ObservationId,
        minimal: bool,
        non_percolable: bool,
    ) -> Result<(), String> {
        self.assert_no_dynamic(&id)?;
        let property =
            DynProperty::mk_trap_space(name, dataset, observation, minimal, non_percolable)?;
        self.dyn_properties.insert(id, property);
        Ok(())
    }

    /// Add a new `DynProperty` instance describing existence of a trajectory corresponding to
    /// observations from a given observation (in the given order).
    pub fn add_dyn_trajectory(
        &mut self,
        id: DynPropertyId,
        name: &str,
        dataset: DatasetId,
    ) -> Result<(), String> {
        self.assert_no_dynamic(&id)?;
        let property = DynProperty::mk_trajectory(name, dataset)?;
        self.dyn_properties.insert(id, property);
        Ok(())
    }

    /// Add a new `DynProperty` instance describing the number of existing attractors.
    pub fn add_dyn_attractor_count(
        &mut self,
        id: DynPropertyId,
        name: &str,
        minimal: usize,
        maximal: usize,
    ) -> Result<(), String> {
        self.assert_no_dynamic(&id)?;
        let property = DynProperty::mk_attractor_count(name, minimal, maximal)?;
        self.dyn_properties.insert(id, property);
        Ok(())
    }

    /// Add a new `DynProperty` instance describing the existence of an attractor corresponding to
    /// a corresponding dataset, or some specific observation in it.
    pub fn add_dyn_has_attractor(
        &mut self,
        id: DynPropertyId,
        name: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
    ) -> Result<(), String> {
        self.assert_no_dynamic(&id)?;
        let property = DynProperty::mk_has_attractor(name, dataset, observation)?;
        self.dyn_properties.insert(id, property);
        Ok(())
    }

    /// Add new "generic" `StatProperty` instance directly from a formula, which must be in a
    /// correct format (which is verified).
    pub fn add_stat_generic(
        &mut self,
        id: StatPropertyId,
        name: &str,
        raw_formula: &str,
    ) -> Result<(), String> {
        self.assert_no_static(&id)?;
        let property = StatProperty::mk_generic(name, raw_formula)?;
        self.stat_properties.insert(id, property);
        Ok(())
    }

    /// Add new `StatProperty` instance describing that an input of an update function is essential.
    pub fn add_stat_update_fn_input_essential(
        &mut self,
        id: StatPropertyId,
        name: &str,
        input: VarId,
        target: VarId,
        value: Essentiality,
        context: Option<String>,
    ) -> Result<(), String> {
        self.assert_no_static(&id)?;
        let property =
            StatProperty::mk_update_fn_input_essential(name, input, target, value, context)?;
        self.stat_properties.insert(id, property);
        Ok(())
    }

    /// Add new `StatProperty` instance describing that an input of an update function is monotonic.
    pub fn add_stat_update_fn_input_monotonic(
        &mut self,
        id: StatPropertyId,
        name: &str,
        input: VarId,
        target: VarId,
        value: Monotonicity,
        context: Option<String>,
    ) -> Result<(), String> {
        self.assert_no_static(&id)?;
        let property =
            StatProperty::mk_update_fn_input_monotonic(name, input, target, value, context)?;
        self.stat_properties.insert(id, property);
        Ok(())
    }

    /// Add new `StatProperty` instance describing that an input of an uninterpreted function
    /// is essential.
    pub fn add_stat_fn_input_essential(
        &mut self,
        id: StatPropertyId,
        name: &str,
        input_index: usize,
        target: UninterpretedFnId,
        value: Essentiality,
        context: Option<String>,
    ) -> Result<(), String> {
        self.assert_no_static(&id)?;
        let property =
            StatProperty::mk_fn_input_essential(name, input_index, target, value, context)?;
        self.stat_properties.insert(id, property);
        Ok(())
    }

    /// Add new `StatProperty` instance describing that an input of an uninterpreted function
    /// is monotonic.
    pub fn add_stat_fn_input_monotonic(
        &mut self,
        id: StatPropertyId,
        name: &str,
        input_index: usize,
        target: UninterpretedFnId,
        value: Monotonicity,
        context: Option<String>,
    ) -> Result<(), String> {
        self.assert_no_static(&id)?;
        let property =
            StatProperty::mk_fn_input_monotonic(name, input_index, target, value, context)?;
        self.stat_properties.insert(id, property);
        Ok(())
    }

    /// Set name for given dynamic property.
    pub fn set_dyn_name(&mut self, id: &DynPropertyId, new_name: &str) -> Result<(), String> {
        self.assert_valid_dynamic(id)?;
        let prop = self.dyn_properties.get_mut(id).unwrap();
        prop.set_name(new_name)
    }

    /// Set name for given static property.
    pub fn set_stat_name(&mut self, id: &StatPropertyId, new_name: &str) -> Result<(), String> {
        self.assert_valid_static(id)?;
        let prop = self.stat_properties.get_mut(id).unwrap();
        prop.set_name(new_name)
    }
}

/// Internal assertion utilities.
impl PropertyManager {
    /// **(internal)** Utility method to ensure there is no dynamic property with given ID yet.
    fn assert_no_dynamic(&self, id: &DynPropertyId) -> Result<(), String> {
        if self.is_valid_dyn_property_id(id) {
            Err(format!("Dynamic property with id {id} already exists."))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a dynamic property with given ID.
    fn assert_valid_dynamic(&self, id: &DynPropertyId) -> Result<(), String> {
        if self.is_valid_dyn_property_id(id) {
            Ok(())
        } else {
            Err(format!("Dynamic property with id {id} does not exist."))
        }
    }

    /// **(internal)** Utility method to ensure there is no static property with given ID yet.
    fn assert_no_static(&self, id: &StatPropertyId) -> Result<(), String> {
        if self.is_valid_stat_property_id(id) {
            Err(format!("Static property with id {id} already exists."))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a static property with given ID.
    fn assert_valid_static(&self, id: &StatPropertyId) -> Result<(), String> {
        if self.is_valid_stat_property_id(id) {
            Ok(())
        } else {
            Err(format!("Static property with id {id} does not exist."))
        }
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

    /// Return a valid dynamic property's `DynPropertyId` corresponding to the given str `id`.
    ///
    /// Return `Err` if such property does not exist (and the ID is invalid).
    pub fn get_dyn_prop_id(&self, id: &str) -> Result<DynPropertyId, String> {
        let property_id = DynPropertyId::from_str(id)?;
        if self.is_valid_dyn_property_id(&property_id) {
            return Ok(property_id);
        }
        Err(format!("Dynamic property with ID {id} does not exist."))
    }

    /// Return a `DynProperty` corresponding to a given `DynPropertyId`.
    ///
    /// Return `Err` if such dynamic property does not exist (the ID is invalid in this context).
    pub fn get_dyn_prop(&self, id: &DynPropertyId) -> Result<&DynProperty, String> {
        let dyn_prop = self
            .dyn_properties
            .get(id)
            .ok_or(format!("Dynamic property with ID {id} does not exist."))?;
        Ok(dyn_prop)
    }

    /// Return a valid static property's `StatPropertyId` corresponding to the given str `id`.
    ///
    /// Return `Err` if such property does not exist (and the ID is invalid).
    pub fn get_stat_prop_id(&self, id: &str) -> Result<StatPropertyId, String> {
        let property_id = StatPropertyId::from_str(id)?;
        if self.is_valid_stat_property_id(&property_id) {
            return Ok(property_id);
        }
        Err(format!("Static property with ID {id} does not exist."))
    }

    /// Return a `StatProperty` corresponding to a given `StatPropertyId`.
    ///
    /// Return `Err` if such static property does not exist (the ID is invalid in this context).
    pub fn get_stat_prop(&self, id: &StatPropertyId) -> Result<&StatProperty, String> {
        let stat_prop = self
            .stat_properties
            .get(id)
            .ok_or(format!("Static property with ID {id} does not exist."))?;
        Ok(stat_prop)
    }
}
