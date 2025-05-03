use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::{AeonError, DynError};
use crate::sketchbook::data_structs::{
    ChangeIdData, LayoutNodeData, LayoutNodeDataPrototype, ModelData, VariableData,
    VariableWithLayoutData,
};
use crate::sketchbook::event_utils::{
    make_reversible, mk_model_event, mk_model_state_change, mk_stat_prop_event,
};
use crate::sketchbook::ids::VarId;
use crate::sketchbook::layout::NodePosition;
use crate::sketchbook::model::{ModelState, UpdateFn, Variable};
use crate::sketchbook::JsonSerde;

/* Constants for event path segments in `ModelState` related to variables. */

// add new propared variable (+ and potentially change its position)
const ADD_VAR_PATH: &str = "add";
// add new default variable
const ADD_DEFAULT_VAR_PATH: &str = "add_default";
// add new variable (without any additional changes)
const ADD_RAW_VAR_PATH: &str = "add_raw";
// remove variable (+ removing all its regulations and so on)
const REMOVE_VAR_PATH: &str = "remove";
// set variable's data (name and annotation)
const SET_DATA_PATH: &str = "set_data";
// set variable's id (+ update all static props)
const SET_ID_PATH: &str = "set_id";
// set variable's id
const SET_ID_RAW_PATH: &str = "set_id_raw";
// set variable's update fn
const SET_UPDATE_FN_PATH: &str = "set_update_fn";

/// Implementation for events related to `variables` of the model.
impl ModelState {
    /// Perform events related to `variables` component of this `ModelState`.
    pub(super) fn perform_variable_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/variable";

        // there is either adding of a new variable, or editing/removing of an existing one
        // when adding new variable, the `at_path` is just ["add"], ["add_default"] or ["add_raw"]
        // when editing existing variable, the `at_path` is ["var_id", "<action>"]

        // adding default version of variable (automatically generated ID, name, empty function)
        // also handles the positioning of the variable
        if Self::starts_with(ADD_DEFAULT_VAR_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_default_variable(event)
        // raw event of adding variable, atomic (no event restart with re-positioning sub-events
        // or anything like that)
        } else if Self::starts_with(ADD_RAW_VAR_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_variable_raw(event)
        // adding variable with all sub-fields given in the event
        // also handles the positioning of the variable
        } else if Self::starts_with(ADD_VAR_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_variable(event)
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let var_id_str = at_path.first().unwrap();
            let var_id = self.get_var_id(var_id_str)?;
            self.event_modify_variable(event, &at_path[1..], var_id)
        }
    }

    /// Perform event of adding a new `variable` component to this `ModelState`.
    /// This expects that the variable was already prepared elsewhere (i.e., its ID and other
    /// fields are already known).
    ///
    /// This event will be broken into sub-events (raw addition of the variable, and re-positioning).
    pub(super) fn event_add_variable(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/variable";

        // get payload components and perform the event
        let payload = Self::clone_payload_str(event, component_name)?;
        let variable_with_pos_data = VariableWithLayoutData::from_json_str(payload.as_str())?;

        // must add variable and then change its position
        let mut event_list = Vec::new();

        // the event of adding the raw variable itself
        let variable_data = variable_with_pos_data.variable.clone();
        let add_event =
            mk_model_event(&["variable", "add_raw"], Some(&variable_data.to_json_str()));
        event_list.push(add_event);

        // update the position in given layouts
        for l_node in variable_with_pos_data.layouts {
            let at_path = ["layout", l_node.layout.as_str(), "update_position"];
            let payload = LayoutNodeData::new(
                l_node.layout.as_str(),
                variable_data.id.as_str(),
                l_node.px,
                l_node.py,
            )
            .to_json_str();
            let move_event = mk_model_event(&at_path, Some(&payload));
            event_list.push(move_event)
        }
        event_list.reverse(); // has to be reversed
        Ok(Consumed::Restart(event_list))
    }

    /// Perform event of adding a new `variable` component to this `ModelState`.
    ///
    /// The field values will be generated or predefined ("default") constants will be used. New ID
    /// will be generated, the same string will be used for its name, and variable will have empty
    /// update function.
    ///
    /// This event will be broken into sub-events (raw addition of the variable, and re-positioning).
    pub(super) fn event_add_default_variable(
        &mut self,
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/variable";
        let payload = Self::clone_payload_str(event, component_name)?;
        let pos_data: Vec<LayoutNodeDataPrototype> = serde_json::from_str(&payload).unwrap();

        // start indexing at 1
        let var_id = self.generate_var_id("var", Some(1));
        let variable = Variable::new(var_id.as_str())?;
        let empty_update = UpdateFn::default();
        let variable_data = VariableData::from_var(&var_id, &variable, &empty_update);

        // must add variable and then change its position
        let mut event_list = Vec::new();

        // the event of adding the raw variable itself
        let add_event =
            mk_model_event(&["variable", "add_raw"], Some(&variable_data.to_json_str()));
        event_list.push(add_event);

        // update the position in given layouts
        for l_node in pos_data {
            let at_path = ["layout", l_node.layout.as_str(), "update_position"];
            let payload = LayoutNodeData::new(
                l_node.layout.as_str(),
                variable_data.id.as_str(),
                l_node.px,
                l_node.py,
            )
            .to_json_str();
            let move_event = mk_model_event(&at_path, Some(&payload));
            event_list.push(move_event)
        }
        event_list.reverse();
        Ok(Consumed::Restart(event_list))
    }

    /// Perform event of adding a new `variable` component to this `ModelState`.
    /// This is an atomic event (only adds variable, already expects layout positioning to
    /// happen elsewhere).
    pub(super) fn event_add_variable_raw(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/variable";

        // get payload components and perform the event
        let payload = Self::clone_payload_str(event, component_name)?;
        let variable_data = VariableData::from_json_str(payload.as_str())?;
        self.add_var_by_str(
            &variable_data.id,
            &variable_data.name,
            &variable_data.annotation,
        )?;

        // prepare the state-change and reverse event (which is a remove event)
        let state_change = mk_model_state_change(&["variable", "add"], &variable_data);
        let reverse_at_path = ["variable", &variable_data.id, "remove"];
        let reverse_event = mk_model_event(&reverse_at_path, None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of modifying or removing existing `variable` component of this `ModelState`.
    pub(super) fn event_modify_variable(
        &mut self,
        event: &Event,
        at_path: &[&str],
        var_id: VarId,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/variable";

        if Self::starts_with(REMOVE_VAR_PATH, at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // First step - check that variable can be safely deleted, i.e., it is not contained in
            // any update function's expression.
            // Note this check is performed also later by the manager, we just want to detect this ASAP.
            if self.is_var_contained_in_expressions(&var_id) {
                return AeonError::throw(format!(
                    "Cannot remove variable `{var_id}`, it is still contained in some update functions."
                ));
            }

            // To remove a variable, all its regulations must be already removed, and it must be at default position
            // in each layout. If it is not the case, we must break this event down into smaller ones to ensure that we can
            // undo this operation later. We prepare a set of events to remove all its regulations, and move the node to
            // default position, and then remove the variable atomically (all as separate undo-able events).

            let targets = self.targets(&var_id)?;
            let regulators = self.regulators(&var_id)?;
            let needs_to_move = self.layouts.iter().fold(true, |acc, (_, l)| {
                acc && (l.get_node_position(&var_id).unwrap() != &NodePosition(0., 0.))
            });

            if regulators.is_empty() && targets.is_empty() && !needs_to_move {
                // save the variable's data for reverse event
                let var_data = VariableData::from_var(
                    &var_id,
                    self.get_variable(&var_id)?,
                    self.get_update_fn(&var_id)?,
                );
                // perform the event, prepare the state-change variant (move id from path to payload)
                self.remove_var(&var_id)?;
                let state_change = mk_model_state_change(&["variable", "remove"], &var_data);

                // prepare the reverse event
                let payload = var_data.to_json_str();
                let reverse_event = mk_model_event(&["variable", "add_raw"], Some(&payload));
                Ok(make_reversible(state_change, event, reverse_event))
            } else {
                let mut event_list = Vec::new();
                event_list.push(event.clone());
                for l_id in self.layouts.keys() {
                    let at_path = ["layout", l_id.as_str(), "update_position"];
                    let payload =
                        LayoutNodeData::new(l_id.as_str(), var_id.as_str(), 0., 0.).to_json_str();
                    let move_event = mk_model_event(&at_path, Some(&payload));
                    event_list.push(move_event)
                }
                for reg in regulators {
                    let at_path = ["regulation", reg.as_str(), var_id.as_str(), "remove"];
                    let remove_event = mk_model_event(&at_path, None);
                    event_list.push(remove_event)
                }
                for target in targets {
                    // if both target and regulator is the same var (self-loop), it was already removed
                    if var_id == *target {
                        continue;
                    }
                    let at_path = ["regulation", var_id.as_str(), target.as_str(), "remove"];
                    let remove_event = mk_model_event(&at_path, None);
                    event_list.push(remove_event)
                }
                Ok(Consumed::Restart(event_list))
            }
        } else if Self::starts_with(SET_DATA_PATH, at_path).is_some() {
            // get the payload - string with modified variable data
            let payload = Self::clone_payload_str(event, component_name)?;
            let new_data = VariableData::from_json_str(&payload)?;
            let new_var = new_data.to_var()?;
            let original_var = self.get_variable(&var_id)?;
            let original_data =
                VariableData::from_var(&var_id, original_var, self.get_update_fn(&var_id)?);
            if &new_var == original_var {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_raw_var(&var_id, new_var)?;
            let new_var = self.get_variable(&var_id)?;
            let new_data = VariableData::from_var(&var_id, new_var, self.get_update_fn(&var_id)?);
            let state_change = mk_model_state_change(&["variable", "set_data"], &new_data);

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(original_data.to_json_str());
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_ID_PATH, at_path).is_some() {
            // get the payload - a string for the "new_id"
            let new_id = Self::clone_payload_str(event, component_name)?;
            if var_id.as_str() == new_id.as_str() {
                return Ok(Consumed::NoChange);
            }

            // now we must handle the event itself, and all potential static property changes
            let mut event_list = Vec::new();
            // the raw event of changing the var id (payload stays the same)
            let var_id_event_path = ["variable", var_id.as_str(), "set_id_raw"];
            let var_id_event = mk_model_event(&var_id_event_path, Some(&new_id));
            event_list.push(var_id_event);

            // event for modifying all affected static properties (we do it via a single special event)
            // note we have checked that `var_id` and `new_id` are different
            let id_change_data = ChangeIdData::new(var_id.as_str(), &new_id).to_json_str();
            let prop_event = mk_stat_prop_event(&["set_var_id_everywhere"], Some(&id_change_data));
            event_list.push(prop_event);
            event_list.reverse(); // has to be reversed
            Ok(Consumed::Restart(event_list))
        } else if Self::starts_with(SET_ID_RAW_PATH, at_path).is_some() {
            // get the payload - string for "new_id"
            let new_id = Self::clone_payload_str(event, component_name)?;
            if var_id.as_str() == new_id.as_str() {
                return Ok(Consumed::NoChange);
            }

            // perform the ID change (which can modify many parts of the model)
            self.set_var_id_by_str(var_id.as_str(), new_id.as_str())?;

            // This event is a bit special - since variable ID change can affect many parts of the model
            // (update fns, regulations, layout, ...), the event returns the whole updated model data to the FE.
            let model_data = ModelData::from_model(self);
            let state_change = mk_model_state_change(&["variable", "set_id"], &model_data);

            // prepare the reverse event (the reverse event is as usual)
            let reverse_at_path = ["variable", new_id.as_str(), "set_id_raw"];
            let reverse_event = mk_model_event(&reverse_at_path, Some(var_id.as_str()));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_UPDATE_FN_PATH, at_path).is_some() {
            // get the payload - string for "new_expression"
            let new_expression = Self::clone_payload_str(event, component_name)?;
            let original_expression = self.get_update_fn(&var_id)?.to_string();
            // actually, this check is not that relevant, as the expressions might be "normalized" during parsing
            if new_expression == original_expression {
                return Ok(Consumed::NoChange);
            }

            // perform the event and check (again) that the new parsed version is different than the original
            self.set_update_fn(&var_id, new_expression.as_str())?;
            let new_update_fn = self.get_update_fn(&var_id)?;
            let var_data =
                VariableData::from_var(&var_id, self.get_variable(&var_id)?, new_update_fn);
            if new_update_fn.get_fn_expression() == original_expression {
                return Ok(Consumed::NoChange);
            }

            // prepare state-change and reverse events
            let state_change = mk_model_state_change(&["variable", "set_update_fn"], &var_data);
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(original_expression);
            Ok(make_reversible(state_change, event, reverse_event))
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }
}
