use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::{ChangeIdData, DynPropertyData, StatPropertyData};
use crate::sketchbook::event_utils::{
    make_refresh_event, make_reversible, mk_dyn_prop_event, mk_dyn_prop_state_change,
    mk_stat_prop_event, mk_stat_prop_state_change,
};
use crate::sketchbook::ids::{DynPropertyId, StatPropertyId, UninterpretedFnId, VarId};
use crate::sketchbook::properties::dynamic_props::SimpleDynPropertyType;
use crate::sketchbook::properties::static_props::SimpleStatPropertyType;
use crate::sketchbook::properties::{DynProperty, PropertyManager, StatProperty};
use crate::sketchbook::JsonSerde;

/* Constants for event path segments for various events. */

// events regarding dynamic properties
const DYNAMIC_PATH: &str = "dynamic";
// events regarding static properties
const STATIC_PATH: &str = "static";
// add a new prepared property
const ADD_PATH: &str = "add";
// add a default variant of a property
const ADD_DEFAULT_PATH: &str = "add_default";
// remove a property
const REMOVE_PATH: &str = "remove";
// set ID of a property
const SET_ID_PATH: &str = "set_id";
// change variable ID in all static properties referencing that variable
const SET_VAR_ID_EVERYWHERE_PATH: &str = "set_var_id_everywhere";
// change function ID in all static properties referencing that function
const SET_FN_ID_EVERYWHERE_PATH: &str = "set_fn_id_everywhere";
// set content of a property
const SET_CONTENT_PATH: &str = "set_content";
// refresh all dynamic properties
const GET_ALL_DYNAMIC_PATH: &str = "get_all_dynamic";
// refresh all static properties
const GET_ALL_STATIC_PATH: &str = "get_all_static";

impl SessionHelper for PropertyManager {}

impl SessionState for PropertyManager {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        let component_name = "properties";

        // the first part of the `at_path` is always "static" or "dynamic"
        // when adding new property, the `at_path` continues with "add" (or "add_default")
        // when editing existing properties, the `at_path` continues with "property_id" and "action"

        match at_path.first() {
            Some(&DYNAMIC_PATH) => {
                let at_path = &at_path[1..];
                if Self::starts_with(ADD_DEFAULT_PATH, at_path).is_some() {
                    Self::assert_path_length(at_path, 1, component_name)?;
                    self.event_add_default_dynamic(event)
                } else if Self::starts_with(ADD_PATH, at_path).is_some() {
                    Self::assert_path_length(at_path, 1, component_name)?;
                    self.event_add_dynamic(event)
                } else {
                    Self::assert_path_length(at_path, 2, component_name)?;
                    let prop_id_str = at_path.first().unwrap();
                    let prop_id = self.get_dyn_prop_id(prop_id_str)?;
                    self.event_modify_dynamic(event, &at_path[1..], prop_id)
                }
            }
            Some(&STATIC_PATH) => {
                let at_path = &at_path[1..];
                if Self::starts_with(ADD_DEFAULT_PATH, at_path).is_some() {
                    Self::assert_path_length(at_path, 1, component_name)?;
                    self.event_add_default_static(event)
                } else if Self::starts_with(ADD_PATH, at_path).is_some() {
                    Self::assert_path_length(at_path, 1, component_name)?;
                    self.event_add_static(event)
                } else if Self::starts_with(SET_VAR_ID_EVERYWHERE_PATH, at_path).is_some() {
                    Self::assert_path_length(at_path, 1, component_name)?;
                    // get the payload - json string encoding the ID change data
                    let payload = Self::clone_payload_str(event, component_name)?;
                    let change_id_data = ChangeIdData::from_json_str(&payload)?;
                    let old_var_id = VarId::new(&change_id_data.original_id)?;
                    let new_var_id = VarId::new(&change_id_data.new_id)?;

                    // change values of all properties that reference this variable (ignoring the rest)
                    for (_, prop) in self.stat_properties.iter_mut() {
                        let _ = prop.set_var_id_if_present(old_var_id.clone(), new_var_id.clone());
                    }
                    self.make_generated_reg_prop_ids_consistent().unwrap(); // this is okay to unwrap here

                    // the state change is just a list of all static properties
                    let mut properties_list: Vec<StatPropertyData> = self
                        .stat_properties
                        .iter()
                        .map(|(id, prop)| StatPropertyData::from_property(id, prop))
                        .collect();
                    properties_list.sort_by(|a, b| a.id.cmp(&b.id));
                    let state_change = Event {
                        path: vec![
                            "sketch".to_string(),
                            "properties".to_string(),
                            "static".to_string(),
                            "all_static_updated".to_string(),
                        ],
                        payload: Some(serde_json::to_string(&properties_list)?),
                    };

                    // prepare the reverse event (setting the original ID back)
                    let reverse_id_change_data =
                        ChangeIdData::new(&change_id_data.new_id, &change_id_data.original_id);
                    let payload = reverse_id_change_data.to_json_str();
                    let reverse_event =
                        mk_stat_prop_event(&[SET_VAR_ID_EVERYWHERE_PATH], Some(&payload));

                    Ok(make_reversible(state_change, event, reverse_event))
                } else if Self::starts_with(SET_FN_ID_EVERYWHERE_PATH, at_path).is_some() {
                    Self::assert_path_length(at_path, 1, component_name)?;
                    // get the payload - json string encoding the ID change data
                    let payload = Self::clone_payload_str(event, component_name)?;
                    let change_id_data = ChangeIdData::from_json_str(&payload)?;
                    let old_fn_id = UninterpretedFnId::new(&change_id_data.original_id)?;
                    let new_fn_id = UninterpretedFnId::new(&change_id_data.new_id)?;

                    // change values of all properties that reference this function (ignoring the rest)
                    for (_, prop) in self.stat_properties.iter_mut() {
                        let _ = prop.set_fn_id_if_present(old_fn_id.clone(), new_fn_id.clone());
                    }
                    self.make_generated_fn_prop_ids_consistent().unwrap(); // this is okay to unwrap here

                    // the state change is just a list of all static properties
                    let mut properties_list: Vec<StatPropertyData> = self
                        .stat_properties
                        .iter()
                        .map(|(id, prop)| StatPropertyData::from_property(id, prop))
                        .collect();
                    properties_list.sort_by(|a, b| a.id.cmp(&b.id));
                    let state_change = Event {
                        path: vec![
                            "sketch".to_string(),
                            "properties".to_string(),
                            "static".to_string(),
                            "all_static_updated".to_string(),
                        ],
                        payload: Some(serde_json::to_string(&properties_list)?),
                    };

                    // prepare the reverse event (setting the original ID back)
                    let reverse_id_change_data =
                        ChangeIdData::new(&change_id_data.new_id, &change_id_data.original_id);
                    let payload = reverse_id_change_data.to_json_str();
                    let reverse_event =
                        mk_stat_prop_event(&[SET_FN_ID_EVERYWHERE_PATH], Some(&payload));

                    Ok(make_reversible(state_change, event, reverse_event))
                } else {
                    Self::assert_path_length(at_path, 2, component_name)?;
                    let prop_id_str = at_path.first().unwrap();
                    let prop_id = self.get_stat_prop_id(prop_id_str)?;
                    self.event_modify_static(event, &at_path[1..], prop_id)
                }
            }
            _ => Self::invalid_path_error_generic(at_path),
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        let component_name = "properties";

        // currently three options: get all datasets, a single dataset, a single observation
        match at_path.first() {
            Some(&GET_ALL_DYNAMIC_PATH) => {
                Self::assert_path_length(at_path, 1, component_name)?;
                let mut properties_list: Vec<DynPropertyData> = self
                    .dyn_properties
                    .iter()
                    .map(|(id, prop)| DynPropertyData::from_property(id, prop))
                    .collect();
                // return the list sorted, so that it is deterministic
                properties_list.sort_by(|a, b| a.id.cmp(&b.id));
                make_refresh_event(full_path, properties_list)
            }
            Some(&GET_ALL_STATIC_PATH) => {
                Self::assert_path_length(at_path, 1, component_name)?;
                let mut properties_list: Vec<StatPropertyData> = self
                    .stat_properties
                    .iter()
                    .map(|(id, prop)| StatPropertyData::from_property(id, prop))
                    .collect();
                // return the list sorted, so that it is deterministic
                properties_list.sort_by(|a, b| a.id.cmp(&b.id));
                make_refresh_event(full_path, properties_list)
            }
            _ => Self::invalid_path_error_generic(at_path),
        }
    }
}

/// Implementation for events related to modifying `dynamic` properties.
impl PropertyManager {
    /// Perform event of adding a new `dynamic property` to this `PropertyManager`.
    pub(super) fn event_add_dynamic(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "properties/dynamic";

        // get payload components and perform the event
        let payload = Self::clone_payload_str(event, component_name)?;
        let prop_data = DynPropertyData::from_json_str(payload.as_str())?;
        let property = prop_data.to_property()?;
        self.add_dynamic_by_str(&prop_data.id, property)?;

        // prepare the state-change and reverse event (which is a remove event)
        let reverse_event = mk_dyn_prop_event(&[&prop_data.id, "remove"], None);
        Ok(make_reversible(event.clone(), event, reverse_event))
    }

    /// Perform event of adding a new DEFAULT `dynamic property` of given variant
    /// to this `PropertyManager`.
    pub(super) fn event_add_default_dynamic(
        &mut self,
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "properties/dynamic";

        // get payload (simplified property type)
        let payload = Self::clone_payload_str(event, component_name)?;
        let prop_type = SimpleDynPropertyType::from_json_str(payload.as_str())?;

        let property = DynProperty::default(prop_type);
        // start indexing at 1
        let prop_id = self.generate_dyn_property_id("dynamic", Some(1));
        let prop_data = DynPropertyData::from_property(&prop_id, &property);

        // actually add the property
        self.add_dynamic_by_str(&prop_data.id, property)?;

        // prepare the state-change (which is add event) and reverse event (which is a remove event)
        let state_change = mk_dyn_prop_state_change(&["add"], &prop_data);
        let reverse_event = mk_dyn_prop_event(&[&prop_data.id, "remove"], None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of modifying or removing existing `dynamic property` of this
    /// `PropertyManager`.
    pub(super) fn event_modify_dynamic(
        &mut self,
        event: &Event,
        at_path: &[&str],
        prop_id: DynPropertyId,
    ) -> Result<Consumed, DynError> {
        let component_name = "properties/dynamic";

        if Self::starts_with(REMOVE_PATH, at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // save the original property data for state change and reverse event
            let original_prop = self.get_dyn_prop(&prop_id)?.clone();
            let prop_data = DynPropertyData::from_property(&prop_id, &original_prop);

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_dynamic(&prop_id)?;
            let state_change = mk_dyn_prop_state_change(&["remove"], &prop_data);

            // prepare the reverse 'add' event (path has no ids, all info carried by payload)
            let payload = prop_data.to_json_str();
            let reverse_event = mk_dyn_prop_event(&["add"], Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_ID_PATH, at_path).is_some() {
            // get the payload - string for "new_id"
            let new_id = Self::clone_payload_str(event, component_name)?;
            if prop_id.as_str() == new_id.as_str() {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_dyn_id_by_str(prop_id.as_str(), new_id.as_str())?;
            let id_change_data = ChangeIdData::new(prop_id.as_str(), new_id.as_str());
            let state_change = mk_dyn_prop_state_change(&["set_id"], &id_change_data);

            // prepare the reverse event (setting the original ID back)
            let payload = prop_id.as_str();
            let reverse_event = mk_dyn_prop_event(&[new_id.as_str(), "set_id"], Some(payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_CONTENT_PATH, at_path).is_some() {
            // get the payload - json string encoding a new property data
            let payload = Self::clone_payload_str(event, component_name)?;
            let new_property_data = DynPropertyData::from_json_str(&payload)?;
            let new_property = new_property_data.to_property()?;
            let orig_property = self.get_dyn_prop(&prop_id)?;
            if orig_property == &new_property {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            let orig_prop_data = DynPropertyData::from_property(&prop_id, orig_property);
            self.swap_dyn_content(&prop_id, new_property)?;
            let state_change = mk_dyn_prop_state_change(&["set_content"], &new_property_data);

            // prepare the reverse event (setting the original ID back)
            let reverse_at_path = [prop_id.as_str(), "set_content"];
            let payload = orig_prop_data.to_json_str();
            let reverse_event = mk_dyn_prop_event(&reverse_at_path, Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }
}

/// Implementation for events related to modifying `static` properties.
impl PropertyManager {
    /// Perform event of adding a new `static property` to this `PropertyManager`.
    pub(super) fn event_add_static(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "properties/static";

        // get payload components and perform the event
        let payload = Self::clone_payload_str(event, component_name)?;
        let prop_data = StatPropertyData::from_json_str(payload.as_str())?;
        let property = prop_data.to_property()?;
        self.add_static_by_str(&prop_data.id, property)?;

        // prepare the state-change and reverse event (which is a remove event)
        let reverse_event = mk_stat_prop_event(&[&prop_data.id, "remove"], None);
        Ok(make_reversible(event.clone(), event, reverse_event))
    }

    /// Perform event of adding a new DEFAULT `static property` of given variant
    /// to this `PropertyManager`.
    pub(super) fn event_add_default_static(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "properties/static";

        // get payload (simplified property type)
        let payload = Self::clone_payload_str(event, component_name)?;
        let prop_type = SimpleStatPropertyType::from_json_str(payload.as_str())?;

        let property = StatProperty::default(prop_type);
        // start indexing at 1
        let prop_id = self.generate_stat_property_id("static", Some(1));
        let prop_data = StatPropertyData::from_property(&prop_id, &property);

        // actually add the property
        self.add_static_by_str(&prop_data.id, property)?;

        // prepare the state-change (which is add event) and reverse event (which is a remove event)
        let state_change = mk_stat_prop_state_change(&["add"], &prop_data);
        let reverse_event = mk_stat_prop_event(&[&prop_data.id, "remove"], None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of modifying or removing existing `static property` of this
    /// `PropertyManager`.
    pub(super) fn event_modify_static(
        &mut self,
        event: &Event,
        at_path: &[&str],
        prop_id: StatPropertyId,
    ) -> Result<Consumed, DynError> {
        let component_name = "properties/static";

        if Self::starts_with(REMOVE_PATH, at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // save the original property data for state change and reverse event
            let original_prop = self.get_stat_prop(&prop_id)?.clone();
            let prop_data = StatPropertyData::from_property(&prop_id, &original_prop);

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_static(&prop_id)?;
            let state_change = mk_stat_prop_state_change(&["remove"], &prop_data);

            // prepare the reverse 'add' event (path has no ids, all info carried by payload)
            let payload = prop_data.to_json_str();
            let reverse_event = mk_stat_prop_event(&["add"], Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_ID_PATH, at_path).is_some() {
            // get the payload - string for "new_id"
            let new_id = Self::clone_payload_str(event, component_name)?;
            if prop_id.as_str() == new_id.as_str() {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_stat_id_by_str(prop_id.as_str(), new_id.as_str())?;
            let id_change_data = ChangeIdData::new(prop_id.as_str(), new_id.as_str());
            let state_change = mk_stat_prop_state_change(&["set_id"], &id_change_data);

            // prepare the reverse event (setting the original ID back)
            let payload = prop_id.as_str();
            let reverse_event = mk_stat_prop_event(&[new_id.as_str(), "set_id"], Some(payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_CONTENT_PATH, at_path).is_some() {
            // get the payload - json string encoding a new property data
            let payload = Self::clone_payload_str(event, component_name)?;
            let new_property_data = StatPropertyData::from_json_str(&payload)?;
            let new_property = new_property_data.to_property()?;
            let orig_property = self.get_stat_prop(&prop_id)?;
            if orig_property == &new_property {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            let orig_prop_data = StatPropertyData::from_property(&prop_id, orig_property);
            self.swap_stat_content(&prop_id, new_property)?;
            let state_change = mk_stat_prop_state_change(&["set_content"], &new_property_data);

            // prepare the reverse event (setting the original ID back)
            let reverse_at_path = [prop_id.as_str(), "set_content"];
            let payload = orig_prop_data.to_json_str();
            let reverse_event = mk_stat_prop_event(&reverse_at_path, Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }
}
