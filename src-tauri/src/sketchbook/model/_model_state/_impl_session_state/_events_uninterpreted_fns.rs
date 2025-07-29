use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::{AeonError, DynError};
use crate::sketchbook::data_structs::{
    ChangeArgEssentialData, ChangeArgMonotoneData, ChangeIdData, ModelData, StatPropertyData,
    UninterpretedFnData,
};
use crate::sketchbook::event_utils::{
    make_reversible, mk_model_event, mk_model_state_change, mk_stat_prop_event,
};
use crate::sketchbook::ids::UninterpretedFnId;
use crate::sketchbook::model::{
    Essentiality, FnArgumentProperty, ModelState, Monotonicity, UninterpretedFn,
};
use crate::sketchbook::properties::shortcuts::*;
use crate::sketchbook::properties::StatProperty;
use crate::sketchbook::JsonSerde;

/* Constants for event path segments in `ModelState` related to uninterpreted functions. */

// add function, and also propagate changes into static properties
const ADD_FN_PATH: &str = "add";
// add function (without additional changes to static properties)
const ADD_RAW_FN_PATH: &str = "add_raw";
// add a new default function (does not require changes in static properties)
const ADD_DEFAULT_FN_PATH: &str = "add_default";
// remove particular function, and also propagate changes into static properties
const REMOVE_FN_PATH: &str = "remove";
// remove particular function (without additional changes to static properties)
const REMOVE_RAW_FN_PATH: &str = "remove_raw";
// set function's (meta)data (name, annotation)
const SET_DATA_PATH: &str = "set_data";
// set function's ID, and also propagate changes into static properties
const SET_ID_RAW_PATH: &str = "set_id_raw";
// set function's ID (without additional changes to static properties)
const SET_ID_PATH: &str = "set_id";
// set function's arity (without additional changes to static properties)
const SET_ARITY_RAW_PATH: &str = "set_arity_raw";
// set function's arity, and also propagate changes into static properties
const SET_ARITY_PATH: &str = "set_arity";
// set function's expression
const SET_EXPRESSION_PATH: &str = "set_expression";
// set function's monotonicity, and also propagate changes into static properties
const SET_MONOTONICITY_RAW_PATH: &str = "set_monotonicity_raw";
// set function's monotonicity (without additional changes to static properties)
const SET_MONOTONICITY_PATH: &str = "set_monotonicity";
// set function's essentiality, and also propagate changes into static properties
const SET_ESSENTIALITY_RAW_PATH: &str = "set_essentiality_raw";
// set function's essentiality (without additional changes to static properties)
const SET_ESSENTIALITY_PATH: &str = "set_essentiality";

/// Implementation for events related to `uninterpreted functions` of the model.
impl ModelState {
    /// Perform events related to `uninterpreted fns` component of this `ModelState`.
    pub(super) fn perform_uninterpreted_fn_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/uninterpreted_fn";

        // there is either adding additional uninterpreted_fn, or editing/removing some existing one
        // adding new uninterpreted fn can be done using `at_path` ["add"], ["add_raw"] or ["add_default"]
        // when editing existing uninterpreted fn, the `at_path` is ["fn_id", "<action>"]

        if Self::starts_with(ADD_DEFAULT_FN_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_default_uninterpreted_fn(event)
        } else if Self::starts_with(ADD_FN_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_uninterpreted_fn(event)
        } else if Self::starts_with(ADD_RAW_FN_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_uninterpreted_fn_raw(event)
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let fn_id_str = at_path.first().unwrap();
            let fn_id = self.get_uninterpreted_fn_id(fn_id_str)?;

            self.event_modify_uninterpreted_fn(event, &at_path[1..], fn_id)
        }
    }

    /// Perform event of adding a new `uninterpreted function`` to this `ModelState`, and
    /// also add corresponding static properties.
    ///
    /// This breaks the event down into atomic events - first to create corresponding static
    /// properties, and then to make the function itself.
    pub(super) fn event_add_uninterpreted_fn(
        &mut self,
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/uninterpreted_fn";
        let payload = Self::clone_payload_str(event, component_name)?;
        let fn_data = UninterpretedFnData::from_json_str(payload.as_str())?;
        let fn_id = UninterpretedFnId::new(&fn_data.id)?;

        let mut event_list = Vec::new();

        // create events to add corresponding properties for monotonicity/essentiality in case they
        // are not unknown variants
        for (index, (monotonicity, essentiality)) in fn_data.arguments.iter().enumerate() {
            if *essentiality != Essentiality::Unknown {
                let prop_id = StatProperty::get_fn_input_essentiality_prop_id(&fn_id, index);
                let prop = mk_fn_input_essentiality_prop(&fn_id, index, *essentiality);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event = mk_stat_prop_event(&["add"], Some(&prop_payload));
                event_list.push(prop_event);
            }
            if *monotonicity != Monotonicity::Unknown {
                let prop_id = StatProperty::get_fn_input_monotonicity_prop_id(&fn_id, index);
                let prop = mk_fn_input_monotonicity_prop(&fn_id, index, *monotonicity);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event = mk_stat_prop_event(&["add"], Some(&prop_payload));
                event_list.push(prop_event);
            }
        }

        // and finally, the event of adding the raw function itself (the event list will be
        // reversed, this being the first of the events with all the checks)
        let reg_event = mk_model_event(&["uninterpreted_fn", "add_raw"], Some(&payload));
        event_list.push(reg_event);

        Ok(Consumed::Restart(event_list))
    }

    /// Perform event of adding an `uninterpreted fn` component to this `ModelState`.
    ///
    /// This version is only adding the function, and not the corresponding static properties.
    /// It is expected that `event_add_uninterpreted_fn` is called first, handling the actual
    /// division into this event + event for adding the properties.
    pub(super) fn event_add_uninterpreted_fn_raw(
        &mut self,
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/uninterpreted_fn";

        // parse the payload and perform the event
        let payload = Self::clone_payload_str(event, component_name)?;
        let fn_data = UninterpretedFnData::from_json_str(payload.as_str())?;
        let fn_arguments = fn_data
            .arguments
            .clone()
            .into_iter()
            .map(|(m, e)| FnArgumentProperty::new(e, m))
            .collect();
        self.add_uninterpreted_fn_by_str(
            &fn_data.id,
            &fn_data.name,
            fn_arguments,
            &fn_data.expression,
            &fn_data.annotation,
        )?;

        // prepare the state-change and reverse event (which is a remove event)
        let state_change = mk_model_state_change(&["uninterpreted_fn", "add"], &fn_data);
        let reverse_at_path = ["uninterpreted_fn", &fn_data.id, "remove_raw"];
        let reverse_event = mk_model_event(&reverse_at_path, None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of adding a new "default" `uninterpreted fn` component to this `ModelState`.
    ///
    /// The field values will be newly generated or predefined ("default") constants will be used.
    /// Particularly, new ID will be generated, the same string will be used for its name, and
    /// arity will be zero.
    /// Default function symbols also have no constraints (monotonicity, essentiality, or expression).
    pub(super) fn event_add_default_uninterpreted_fn(
        &mut self,
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/uninterpreted_fn";
        Self::assert_payload_empty(event, component_name)?;

        let arity = 0;
        // start indexing at 1
        let fn_id = self.generate_uninterpreted_fn_id("fn", Some(1));
        let uninterpreted_fn = UninterpretedFn::new_default(fn_id.as_str(), arity)?;
        let fn_data = UninterpretedFnData::from_fn(&fn_id, &uninterpreted_fn);
        self.add_empty_uninterpreted_fn_by_str(&fn_data.id, &fn_data.name, arity)?;

        // prepare the state-change and reverse event (which is a remove event)
        let state_change = mk_model_state_change(&["uninterpreted_fn", "add"], &fn_data);
        let reverse_at_path = ["uninterpreted_fn", &fn_data.id, "remove"];
        let reverse_event = mk_model_event(&reverse_at_path, None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of modifying or removing existing `uninterpreted fn` component of this `ModelState`.
    pub(super) fn event_modify_uninterpreted_fn(
        &mut self,
        event: &Event,
        at_path: &[&str],
        fn_id: UninterpretedFnId,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/uninterpreted_fn";

        if Self::starts_with(REMOVE_FN_PATH, at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // First step - check that function can be safely deleted, i.e., it is not contained in
            // any update/uninterpreted function's expression.
            // Note this check is also performed also later by the manager, we just want to detect this ASAP.
            if self.is_fn_contained_in_expressions(&fn_id) {
                return AeonError::throw(format!(
                    "Cannot remove function `{fn_id}`, it is still contained in some update/uninterpreted function."
                ));
            }

            // To remove a function, all its essentiality/monotonicity properties must also be removed.
            // We break this event down into atomic ones to ensure that we can undo this operation later.
            // We prepare  a set of events to remove all the properties, and then remove the
            // function atomically (all as separate undo-able events).
            let mut event_list = Vec::new();

            // create the event of removing the raw function itself
            // the event list will be reversed, and this will become the last of the sub-events processed
            let fn_event_path = ["uninterpreted_fn", fn_id.as_str(), "remove_raw"];
            let fn_event = mk_model_event(&fn_event_path, None);
            event_list.push(fn_event);

            // events of removing the corresponding properties for monotonicity/essentiality in
            // case it is not unknown variant
            let orig_fn = self.get_uninterpreted_fn(&fn_id)?;
            for (index, argument) in orig_fn.get_all_arguments().iter().enumerate() {
                if argument.essential != Essentiality::Unknown {
                    let prop_id = StatProperty::get_fn_input_essentiality_prop_id(&fn_id, index);
                    let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                    event_list.push(prop_event);
                }
                if argument.monotonicity != Monotonicity::Unknown {
                    let prop_id = StatProperty::get_fn_input_monotonicity_prop_id(&fn_id, index);
                    let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                    event_list.push(prop_event);
                }
            }
            Ok(Consumed::Restart(event_list))
        } else if Self::starts_with(REMOVE_RAW_FN_PATH, at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // perform the event, prepare the state-change variant (move id from path to payload)
            // note that to remove an uninterpreted_fn, it must not be used in any update fn (checked during the call)
            let fn_data = UninterpretedFnData::from_fn(&fn_id, self.get_uninterpreted_fn(&fn_id)?);
            self.remove_uninterpreted_fn(&fn_id)?;
            let state_change = mk_model_state_change(&["uninterpreted_fn", "remove"], &fn_data);

            // prepare the reverse event
            let reverse_at_path = ["uninterpreted_fn", "add_raw"];
            let payload = fn_data.to_json_str();
            let reverse_event = mk_model_event(&reverse_at_path, Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_DATA_PATH, at_path).is_some() {
            // get the payload - string with modified function data
            let payload = Self::clone_payload_str(event, component_name)?;
            let new_data = UninterpretedFnData::from_json_str(&payload)?;
            let new_fn = new_data.to_uninterpreted_fn(self)?;
            let original_fn = self.get_uninterpreted_fn(&fn_id)?;
            let original_data = UninterpretedFnData::from_fn(&fn_id, original_fn);
            if &new_fn == original_fn {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_raw_function(&fn_id, new_fn)?;
            let new_fn = self.get_uninterpreted_fn(&fn_id)?;
            let new_data = UninterpretedFnData::from_fn(&fn_id, new_fn);
            let state_change = mk_model_state_change(&["uninterpreted_fn", "set_data"], &new_data);

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(original_data.to_json_str());
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_ID_PATH, at_path).is_some() {
            // get the payload - string for "new_id"
            let new_id = Self::clone_payload_str(event, component_name)?;
            if fn_id.as_str() == new_id.as_str() {
                return Ok(Consumed::NoChange);
            }

            // now we must handle the event itself, and all potential static property changes
            let mut event_list = Vec::new();
            // the raw event of changing the function id (payload stays the same)
            let fn_id_event_path = ["uninterpreted_fn", fn_id.as_str(), "set_id_raw"];
            let fn_id_event = mk_model_event(&fn_id_event_path, Some(&new_id));
            event_list.push(fn_id_event);

            // event for modifying all affected static properties (we do it via a single special event)
            // note we have checked that `fn_id` and `new_id` are different
            let id_change_data = ChangeIdData::new(fn_id.as_str(), &new_id).to_json_str();
            let prop_event = mk_stat_prop_event(&["set_fn_id_everywhere"], Some(&id_change_data));
            event_list.push(prop_event);
            event_list.reverse(); // has to be reversed
            Ok(Consumed::Restart(event_list))
        } else if Self::starts_with(SET_ID_RAW_PATH, at_path).is_some() {
            // get the payload - string for "new_id"
            let new_id = Self::clone_payload_str(event, component_name)?;
            if fn_id.as_str() == new_id.as_str() {
                return Ok(Consumed::NoChange);
            }

            // perform the ID change (which can modify many parts of the model)
            self.set_uninterpreted_fn_id_by_str(fn_id.as_str(), new_id.as_str())?;

            // This event is a bit special - since variable ID change can affect many parts of the model
            // (update fns, regulations, layout, ...), the event returns the whole updated model data to the FE.
            let model_data = ModelData::from_model(self);
            let state_change = mk_model_state_change(&["uninterpreted_fn", "set_id"], &model_data);

            // prepare the reverse event
            let reverse_at_path = ["uninterpreted_fn", new_id.as_str(), "set_id_raw"];
            let reverse_event = mk_model_event(&reverse_at_path, Some(fn_id.as_str()));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_ARITY_PATH, at_path).is_some() {
            // get the payload - string for "new_arity"
            let payload = Self::clone_payload_str(event, component_name)?;
            let new_arity: usize = payload.parse()?;
            let original_arity = self.get_uninterpreted_fn(&fn_id)?.get_arity();
            if new_arity == original_arity {
                return Ok(Consumed::NoChange);
            }

            // If the arity is lowered (removing some function arguments), we may also need to remove
            // their monotonicity/essentiality and corresponding static properties.
            // Therefore, we break this event down into atomic sub-events in that case.
            // If the arity is raised, we dont need to alter any static properties.
            let mut event_list = Vec::new();

            // start with the event of raw arity setting of then function
            // the event list will be reversed, and this will become the last of the sub-events processed
            let fn_event_path = ["uninterpreted_fn", fn_id.as_str(), "set_arity_raw"];
            let fn_event = mk_model_event(&fn_event_path, Some(&payload));
            event_list.push(fn_event);

            // for arguments that will be removed by arity change, we must remove also the corresponding
            // static properties for monotonicity/essentiality (in case it is not unknown variant)
            let orig_fn = self.get_uninterpreted_fn(&fn_id)?;
            for (index, argument) in orig_fn.get_all_arguments().iter().enumerate() {
                if index < new_arity {
                    // all arguments with index up to `arity` stay unchanged
                    continue;
                }
                if argument.essential != Essentiality::Unknown {
                    // First prepare event to set the essentiality of the argument to unknown
                    // (so that it can be reversed later)
                    let essent_event_path =
                        ["uninterpreted_fn", fn_id.as_str(), "set_essentiality_raw"];
                    let essent_payload =
                        ChangeArgEssentialData::new(index, Essentiality::Unknown).to_json_str();
                    let essent_event = mk_model_event(&essent_event_path, Some(&essent_payload));
                    event_list.push(essent_event);

                    // and also remove the static property
                    let prop_id = StatProperty::get_fn_input_essentiality_prop_id(&fn_id, index);
                    let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                    event_list.push(prop_event);
                }
                if argument.monotonicity != Monotonicity::Unknown {
                    // First prepare event to set the monotonicity of the argument to unknown
                    // (so that it can be reversed later)
                    let monot_event_path =
                        ["uninterpreted_fn", fn_id.as_str(), "set_monotonicity_raw"];
                    let monot_payload =
                        ChangeArgMonotoneData::new(index, Monotonicity::Unknown).to_json_str();
                    let monot_event = mk_model_event(&monot_event_path, Some(&monot_payload));
                    event_list.push(monot_event);

                    // and also remove the static property
                    let prop_id = StatProperty::get_fn_input_monotonicity_prop_id(&fn_id, index);
                    let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                    event_list.push(prop_event);
                }
            }
            Ok(Consumed::Restart(event_list))
        } else if Self::starts_with(SET_ARITY_RAW_PATH, at_path).is_some() {
            // get the payload - string for "new_arity"
            let new_arity: usize = Self::clone_payload_str(event, component_name)?.parse()?;
            let original_arity = self.get_uninterpreted_fn(&fn_id)?.get_arity();
            if new_arity == original_arity {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_uninterpreted_fn_arity(&fn_id, new_arity)?;
            let fn_data = UninterpretedFnData::from_fn(&fn_id, self.get_uninterpreted_fn(&fn_id)?);
            let state_change = mk_model_state_change(&["uninterpreted_fn", "set_arity"], &fn_data);

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(original_arity.to_string());
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_EXPRESSION_PATH, at_path).is_some() {
            // get the payload - string for "expression"
            let new_expression = Self::clone_payload_str(event, component_name)?;
            let original_expression = self
                .get_uninterpreted_fn(&fn_id)?
                .get_fn_expression()
                .to_string();
            // actually, this check is not that relevant, as the expressions might be "normalized" during parsing
            if new_expression == original_expression {
                return Ok(Consumed::NoChange);
            }

            // perform the event and check (again) that the new parsed version is different than the original
            self.set_uninterpreted_fn_expression(&fn_id, new_expression.as_str())?;
            let new_fn = self.get_uninterpreted_fn(&fn_id)?;
            let fn_data = UninterpretedFnData::from_fn(&fn_id, new_fn);
            if fn_data.expression == original_expression {
                return Ok(Consumed::NoChange);
            }

            // prepare the state-change and reverse event
            let state_change =
                mk_model_state_change(&["uninterpreted_fn", "set_expression"], &fn_data);
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(original_expression);
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_MONOTONICITY_PATH, at_path).is_some() {
            // This event is broken down into atomic events - first to potentially create corresponding
            // static property, and then to modify the function itself.

            // get the payload and parse it
            let payload = Self::clone_payload_str(event, component_name)?;
            let change_data = ChangeArgMonotoneData::from_json_str(payload.as_str())?;
            let orig_monotonicity = *self
                .get_uninterpreted_fn(&fn_id)?
                .get_monotonic(change_data.idx);
            let new_monotonicity = change_data.monotonicity;
            if orig_monotonicity == new_monotonicity {
                return Ok(Consumed::NoChange);
            }

            let mut event_list = Vec::new();

            // prepare the event of changing the function monotonicity (payload stays the same)
            let fn_event_path = ["uninterpreted_fn", fn_id.as_str(), "set_monotonicity_raw"];
            let fn_event = mk_model_event(&fn_event_path, Some(&payload));
            event_list.push(fn_event);

            // now the event of modifying/adding/removing corresponding static property
            // note we have checked that orig monotonicity and new monotonicity are different
            let prop_id = StatProperty::get_fn_input_monotonicity_prop_id(&fn_id, change_data.idx);
            if orig_monotonicity == Monotonicity::Unknown {
                // before there was no static prop, now we have to add it
                let prop = mk_fn_input_monotonicity_prop(&fn_id, change_data.idx, new_monotonicity);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event = mk_stat_prop_event(&["add"], Some(&prop_payload));
                event_list.push(prop_event);
            } else if new_monotonicity == Monotonicity::Unknown {
                // before there was a static prop, now we have to remove it
                let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                event_list.push(prop_event);
            } else {
                // there is a static prop, and we must change its monotonicity
                let prop = mk_fn_input_monotonicity_prop(&fn_id, change_data.idx, new_monotonicity);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event =
                    mk_stat_prop_event(&[prop_id.as_str(), "set_content"], Some(&prop_payload));
                event_list.push(prop_event);
            }
            Ok(Consumed::Restart(event_list))
        } else if Self::starts_with(SET_MONOTONICITY_RAW_PATH, at_path).is_some() {
            // get the payload and parse it
            let payload = Self::clone_payload_str(event, component_name)?;
            let change_data = ChangeArgMonotoneData::from_json_str(payload.as_str())?;
            let original_monotonicity = *self
                .get_uninterpreted_fn(&fn_id)?
                .get_monotonic(change_data.idx);
            if original_monotonicity == change_data.monotonicity {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_uninterpreted_fn_monotonicity(
                &fn_id,
                change_data.monotonicity,
                change_data.idx,
            )?;
            let fn_data = UninterpretedFnData::from_fn(&fn_id, self.get_uninterpreted_fn(&fn_id)?);
            let state_change =
                mk_model_state_change(&["uninterpreted_fn", "set_monotonicity"], &fn_data);

            // prepare the reverse event
            let mut reverse_event = event.clone();
            let reverse_change = ChangeArgMonotoneData::new(change_data.idx, original_monotonicity);
            reverse_event.payload = Some(reverse_change.to_json_str());
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_ESSENTIALITY_PATH, at_path).is_some() {
            // This event is broken down into atomic events - first to potentially create corresponding
            // static property, and then to modify the function itself.

            // get the payload and parse it
            let payload = Self::clone_payload_str(event, component_name)?;
            let change_data = ChangeArgEssentialData::from_json_str(payload.as_str())?;
            let orig_essentiality = *self
                .get_uninterpreted_fn(&fn_id)?
                .get_essential(change_data.idx);
            let new_essentiality = change_data.essentiality;
            if orig_essentiality == new_essentiality {
                return Ok(Consumed::NoChange);
            }

            let mut event_list = Vec::new();

            // prepare the event of changing the function essentiality (payload stays the same)
            let fn_event_path = ["uninterpreted_fn", fn_id.as_str(), "set_essentiality_raw"];
            let fn_event = mk_model_event(&fn_event_path, Some(&payload));
            event_list.push(fn_event);

            // now the event of modifying/adding/removing corresponding static property
            // note we have checked that orig essentiality and new essentiality are different
            let prop_id = StatProperty::get_fn_input_essentiality_prop_id(&fn_id, change_data.idx);
            if orig_essentiality == Essentiality::Unknown {
                // before there was no static prop, now we have to add it
                let prop = mk_fn_input_essentiality_prop(&fn_id, change_data.idx, new_essentiality);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event = mk_stat_prop_event(&["add"], Some(&prop_payload));
                event_list.push(prop_event);
            } else if new_essentiality == Essentiality::Unknown {
                // before there was a static prop, now we have to remove it
                let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                event_list.push(prop_event);
            } else {
                // there is a static prop, and we must change its essentiality
                let prop = mk_fn_input_essentiality_prop(&fn_id, change_data.idx, new_essentiality);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event =
                    mk_stat_prop_event(&[prop_id.as_str(), "set_content"], Some(&prop_payload));
                event_list.push(prop_event);
            }
            Ok(Consumed::Restart(event_list))
        } else if Self::starts_with(SET_ESSENTIALITY_RAW_PATH, at_path).is_some() {
            // get the payload and parse it
            let payload = Self::clone_payload_str(event, component_name)?;
            let change_data = ChangeArgEssentialData::from_json_str(payload.as_str())?;
            let original_essentiality = *self
                .get_uninterpreted_fn(&fn_id)?
                .get_essential(change_data.idx);
            if original_essentiality == change_data.essentiality {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_uninterpreted_fn_essentiality(
                &fn_id,
                change_data.essentiality,
                change_data.idx,
            )?;
            let fn_data = UninterpretedFnData::from_fn(&fn_id, self.get_uninterpreted_fn(&fn_id)?);
            let state_change =
                mk_model_state_change(&["uninterpreted_fn", "set_essentiality"], &fn_data);

            // prepare the reverse event
            let mut reverse_event = event.clone();
            let reverse_change =
                ChangeArgEssentialData::new(change_data.idx, original_essentiality);
            reverse_event.payload = Some(reverse_change.to_json_str());
            Ok(make_reversible(state_change, event, reverse_event))
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }
}
