use crate::sketchbook::ids::{
    DatasetId, DynPropertyId, ObservationId, StatPropertyId, UninterpretedFnId, VarId,
};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::dynamic_props::are_same_dyn_variant;
use crate::sketchbook::properties::static_props::{are_same_stat_variant, StatPropertyType};
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
    pub fn add_dynamic(&mut self, id: DynPropertyId, prop: DynProperty) -> Result<(), String> {
        self.assert_no_dynamic(&id)?;
        self.dyn_properties.insert(id, prop);
        Ok(())
    }

    /// Add pre-generated dynamic property with id given by str.
    pub fn add_dynamic_by_str(&mut self, id: &str, prop: DynProperty) -> Result<(), String> {
        let id = DynPropertyId::new(id)?;
        self.add_dynamic(id, prop)
    }

    /// Add pre-generated static property.
    pub fn add_static(&mut self, id: StatPropertyId, prop: StatProperty) -> Result<(), String> {
        self.assert_no_static(&id)?;
        self.stat_properties.insert(id, prop);
        Ok(())
    }

    /// Add pre-generated static property with id given by str.
    pub fn add_static_by_str(&mut self, id: &str, prop: StatProperty) -> Result<(), String> {
        let id = StatPropertyId::new(id)?;
        self.add_static(id, prop)
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

    /// Update dynamic property's sub-field `dataset` where applicable.
    /// If not applicable, return `Err`.
    pub fn set_dyn_dataset(
        &mut self,
        id: &DynPropertyId,
        new_dataset: DatasetId,
    ) -> Result<(), String> {
        self.assert_valid_dynamic(id)?;
        let prop = self.dyn_properties.get_mut(id).unwrap();
        prop.set_dataset(new_dataset)
    }

    /// Update dynamic property's sub-field `observation` where applicable.
    /// If not applicable, return `Err`.
    pub fn set_dyn_observation(
        &mut self,
        id: &DynPropertyId,
        new_obs: ObservationId,
    ) -> Result<(), String> {
        self.assert_valid_dynamic(id)?;
        let prop = self.dyn_properties.get_mut(id).unwrap();
        prop.set_observation(new_obs)
    }

    /// Update generic dynamic property's formula.
    /// If not applicable (different variant), return `Err`.
    pub fn set_dyn_formula(&mut self, id: &DynPropertyId, new_formula: &str) -> Result<(), String> {
        self.assert_valid_dynamic(id)?;
        let prop = self.dyn_properties.get_mut(id).unwrap();
        prop.set_formula(new_formula)
    }

    /// Update dynamic property's sub-field `observation` to None where applicable.
    /// If not applicable, return `Err`.
    pub fn set_dyn_none_observation(&mut self, id: &DynPropertyId) -> Result<(), String> {
        self.assert_valid_dynamic(id)?;
        let prop = self.dyn_properties.get_mut(id).unwrap();
        prop.remove_observation()
    }

    /// Update dynamic property's sub-fields, if the property is of `AttractorCount` variant.
    /// If not applicable, return `Err`.
    pub fn set_dyn_attr_count(
        &mut self,
        id: &DynPropertyId,
        minimal: usize,
        maximal: usize,
    ) -> Result<(), String> {
        self.assert_valid_dynamic(id)?;
        let prop = self.dyn_properties.get_mut(id).unwrap();
        prop.set_attr_count(minimal, maximal)
    }

    /// Update dynamic property's sub-fields, if the property is of `ExistsTrapSpace` variant.
    /// If not applicable, return `Err`.
    pub fn set_dyn_trap_space_details(
        &mut self,
        id: &DynPropertyId,
        is_minimal: bool,
        non_percolable: bool,
    ) -> Result<(), String> {
        self.assert_valid_dynamic(id)?;
        let prop = self.dyn_properties.get_mut(id).unwrap();
        prop.set_trap_space_details(is_minimal, non_percolable)
    }

    /// Update generic static property's formula.
    /// If not applicable (different variant), return `Err`.
    pub fn set_stat_formula(
        &mut self,
        id: &StatPropertyId,
        new_formula: &str,
    ) -> Result<(), String> {
        self.assert_valid_static(id)?;
        let prop = self.stat_properties.get_mut(id).unwrap();
        prop.set_formula(new_formula)
    }

    /// Update static property's sub-field for input variable (of an update fn), where applicable.
    /// If not applicable, return `Err`.
    pub fn set_stat_input_var(
        &mut self,
        id: &StatPropertyId,
        new_var: VarId,
    ) -> Result<(), String> {
        self.assert_valid_static(id)?;
        let prop = self.stat_properties.get_mut(id).unwrap();
        prop.set_input_var(new_var)
    }

    /// Update static property's sub-field for index of input (of an uninterpreted fn),
    /// where applicable. If not applicable, return `Err`.
    pub fn set_stat_input_index(
        &mut self,
        id: &StatPropertyId,
        new_idx: usize,
    ) -> Result<(), String> {
        self.assert_valid_static(id)?;
        let prop = self.stat_properties.get_mut(id).unwrap();
        prop.set_input_index(new_idx)
    }

    /// Update static property's sub-field for target uninterpreted fn, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_stat_target_fn(
        &mut self,
        id: &StatPropertyId,
        new_target: UninterpretedFnId,
    ) -> Result<(), String> {
        self.assert_valid_static(id)?;
        let prop = self.stat_properties.get_mut(id).unwrap();
        prop.set_target_fn(new_target)
    }

    /// Update static property's sub-field for target variable, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_stat_target_var(
        &mut self,
        id: &StatPropertyId,
        new_target: VarId,
    ) -> Result<(), String> {
        self.assert_valid_static(id)?;
        let prop = self.stat_properties.get_mut(id).unwrap();
        prop.set_target_var(new_target)
    }

    /// Update static property's sub-field for monotonicity, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_stat_monotonicity(
        &mut self,
        id: &StatPropertyId,
        monotonicity: Monotonicity,
    ) -> Result<(), String> {
        self.assert_valid_static(id)?;
        let prop = self.stat_properties.get_mut(id).unwrap();
        prop.set_monotonicity(monotonicity)
    }

    /// Update static property's sub-field for essentiality, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_stat_essentiality(
        &mut self,
        id: &StatPropertyId,
        essentiality: Essentiality,
    ) -> Result<(), String> {
        self.assert_valid_static(id)?;
        let prop = self.stat_properties.get_mut(id).unwrap();
        prop.set_essentiality(essentiality)
    }

    /// Update static property's sub-field for context, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_stat_context(&mut self, id: &StatPropertyId, context: String) -> Result<(), String> {
        self.assert_valid_static(id)?;
        let prop = self.stat_properties.get_mut(id).unwrap();
        prop.set_context(context)
    }

    /// Swap content of a dynamic property with given `id`. The ID must be valid identifier.
    /// The variant of the prop. must stay the same (i.e., we only change attributes, not variant).
    pub fn swap_dyn_content(
        &mut self,
        id: &DynPropertyId,
        new_content: DynProperty,
    ) -> Result<(), String> {
        let orig_content = self.get_dyn_prop(id)?;
        if !are_same_dyn_variant(new_content.get_prop_data(), orig_content.get_prop_data()) {
            return Err("Variant of the dynamic property cannot change.".to_string());
        }
        self.dyn_properties.insert(id.clone(), new_content);
        Ok(())
    }

    /// Swap content of a dynamic property with given `id`. The ID must be valid identifier.
    /// The variant of the prop. must stay the same (i.e., we only change attributes, not variant).
    pub fn swap_dyn_content_by_str(
        &mut self,
        id: &str,
        new_content: DynProperty,
    ) -> Result<(), String> {
        let prop_id = DynPropertyId::new(id)?;
        self.swap_dyn_content(&prop_id, new_content)
    }

    /// Swap content of a static property with given `id`. The ID must be valid identifier.
    /// The variant of the prop. must stay the same (i.e., we only change attributes, not variant).
    pub fn swap_stat_content(
        &mut self,
        id: &StatPropertyId,
        new_content: StatProperty,
    ) -> Result<(), String> {
        let orig_content = self.get_stat_prop(id)?;
        if !are_same_stat_variant(new_content.get_prop_data(), orig_content.get_prop_data()) {
            return Err("Variant of the static property cannot change.".to_string());
        }
        self.stat_properties.insert(id.clone(), new_content);
        Ok(())
    }

    /// Swap content of a static property with given `id`. The ID must be valid identifier.
    /// The variant of the prop. must stay the same (i.e., we only change attributes, not variant).
    pub fn swap_stat_content_by_str(
        &mut self,
        id: &str,
        new_content: StatProperty,
    ) -> Result<(), String> {
        let prop_id = StatPropertyId::new(id)?;
        self.swap_stat_content(&prop_id, new_content)
    }

    /// Change ID of a dynamic property.
    pub fn set_dyn_id(
        &mut self,
        original_id: &DynPropertyId,
        new_id: DynPropertyId,
    ) -> Result<(), String> {
        self.assert_valid_dynamic(original_id)?;
        self.assert_no_dynamic(&new_id)?;

        if let Some(property) = self.dyn_properties.remove(original_id) {
            self.dyn_properties.insert(new_id.clone(), property);
        } else {
            panic!("Error when modifying dyn property's id in the property map.");
        }
        Ok(())
    }

    /// Change ID of a dynamic property, with IDs given as string slices.
    pub fn set_dyn_id_by_str(&mut self, original_id: &str, new_id: &str) -> Result<(), String> {
        let original_id = DynPropertyId::new(original_id)?;
        let new_id = DynPropertyId::new(new_id)?;
        self.set_dyn_id(&original_id, new_id)
    }

    /// Change ID of a static property.
    pub fn set_stat_id(
        &mut self,
        original_id: &StatPropertyId,
        new_id: StatPropertyId,
    ) -> Result<(), String> {
        self.assert_valid_static(original_id)?;
        self.assert_no_static(&new_id)?;

        if let Some(property) = self.stat_properties.remove(original_id) {
            self.stat_properties.insert(new_id.clone(), property);
        } else {
            panic!("Error when modifying stat property's id in the property map.");
        }
        Ok(())
    }

    /// Change ID of a static property, with IDs given as string slices.
    pub fn set_stat_id_by_str(&mut self, original_id: &str, new_id: &str) -> Result<(), String> {
        let original_id = StatPropertyId::new(original_id)?;
        let new_id = StatPropertyId::new(new_id)?;
        self.set_stat_id(&original_id, new_id)
    }

    /// Remove dynamic property.
    pub fn remove_dynamic(&mut self, id: &DynPropertyId) -> Result<(), String> {
        self.assert_valid_dynamic(id)?;
        self.dyn_properties.remove(id).unwrap();
        Ok(())
    }

    /// Remove static property.
    pub fn remove_static(&mut self, id: &StatPropertyId) -> Result<(), String> {
        self.assert_valid_static(id)?;
        self.stat_properties.remove(id).unwrap();
        Ok(())
    }

    /// Go through all static properties that are automatically generated from the regulation
    /// graph and make their IDs consistent with the variables they reference.
    ///
    /// This is useful after we change some variable's ID, and need to ensure that all
    /// monotonicity/essentiality properties still have IDs in format `monotonicity_REGULATOR_TARGET`.
    ///
    /// This can be also helpful if we are loading model with potentially invalid IDs (changed by
    /// the user outside of Sketchbook) to fix potential issues.
    pub fn make_generated_reg_prop_ids_consistent(&mut self) {
        // first, collect a list of old-new ID pairs that must be changed
        let mut id_change_list: Vec<(StatPropertyId, StatPropertyId)> = Vec::new();
        for (prop_id, prop) in self.stat_properties.iter() {
            match prop.get_prop_data() {
                StatPropertyType::RegulationEssential(p) => {
                    // this template always has both fields filled, we can unwrap
                    let expected_id = StatProperty::get_reg_essentiality_prop_id(
                        p.input.as_ref().unwrap(),
                        p.target.as_ref().unwrap(),
                    );
                    if prop_id != &expected_id {
                        id_change_list.push((prop_id.clone(), expected_id.clone()));
                    }
                }
                StatPropertyType::RegulationMonotonic(p) => {
                    // this template always has both fields filled, we can unwrap
                    let expected_id = StatProperty::get_reg_monotonicity_prop_id(
                        p.input.as_ref().unwrap(),
                        p.target.as_ref().unwrap(),
                    );
                    if prop_id != &expected_id {
                        id_change_list.push((prop_id.clone(), expected_id.clone()));
                    }
                }
                _ => {}
            }
        }
        // and finally, set the IDs
        for (current_id, new_id) in id_change_list {
            self.set_stat_id(&current_id, new_id).unwrap();
        }
    }

    /// Go through all static properties that are automatically generated from the uninterpreted
    /// function properties and make their IDs consistent with the function and argument they reference.
    ///
    /// This is useful after we change some function's ID, and need to ensure that all monotonicity/essentiality
    /// properties still have IDs in format `fn_monotonicity_FUNCTION_INDEX`.
    ///
    /// This can be also helpful if we are loading model with potentially invalid IDs (changed by
    /// the user outside of Sketchbook) to fix potential issues.
    pub fn make_generated_fn_prop_ids_consistent(&mut self) {
        // first, collect a list of old-new ID pairs that must be changed
        let mut id_change_list: Vec<(StatPropertyId, StatPropertyId)> = Vec::new();
        for (prop_id, prop) in self.stat_properties.iter() {
            match prop.get_prop_data() {
                StatPropertyType::FnInputEssential(p) => {
                    // this template always has both fields filled, we can unwrap
                    let expected_id = StatProperty::get_fn_input_essentiality_prop_id(
                        p.target.as_ref().unwrap(),
                        p.input_index.unwrap(),
                    );
                    if prop_id != &expected_id {
                        id_change_list.push((prop_id.clone(), expected_id.clone()));
                    }
                }
                StatPropertyType::FnInputMonotonic(p) => {
                    // this template always has both fields filled, we can unwrap
                    let expected_id = StatProperty::get_fn_input_monotonicity_prop_id(
                        p.target.as_ref().unwrap(),
                        p.input_index.unwrap(),
                    );
                    if prop_id != &expected_id {
                        id_change_list.push((prop_id.clone(), expected_id.clone()));
                    }
                }
                _ => {}
            }
        }
        // and finally, set the IDs
        for (current_id, new_id) in id_change_list {
            self.set_stat_id(&current_id, new_id).unwrap();
        }
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
