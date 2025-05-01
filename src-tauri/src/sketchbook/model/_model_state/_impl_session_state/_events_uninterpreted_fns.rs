use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::{AeonError, DynError};
use crate::sketchbook::data_structs::{
    ChangeArgEssentialData, ChangeArgMonotoneData, ModelData, UninterpretedFnData,
};
use crate::sketchbook::event_utils::{make_reversible, mk_model_event, mk_model_state_change};
use crate::sketchbook::ids::UninterpretedFnId;
use crate::sketchbook::model::{ModelState, UninterpretedFn};
use crate::sketchbook::JsonSerde;

/* Constants for event path segments in `ModelState` related to uninterpreted functions. */

// add new prepared function
const ADD_FN_PATH: &str = "add";
// add new default function
const ADD_DEFAULT_FN_PATH: &str = "add_default";
// remove particular function
const REMOVE_FN_PATH: &str = "remove";
// set function's (meta)data (name, annotation)
const SET_DATA_PATH: &str = "set_data";
// set function's ID
const SET_ID_PATH: &str = "set_id";
// set function's arity
const SET_ARITY_PATH: &str = "set_arity";
// set function's expression
const SET_EXPRESSION_PATH: &str = "set_expression";
// set monotonicity of function with respect to its argument
const SET_MONOTONICITY_PATH: &str = "set_monotonicity";
// set essentiality of function with respect to its argument
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

        // there is either adding of a new uninterpreted_fn, or editing/removing of an existing one
        // when adding new uninterpreted fn, the `at_path` is just ["add"] or ["add_default"]
        // when editing existing uninterpreted fn, the `at_path` is ["fn_id", "<action>"]

        if Self::starts_with(ADD_DEFAULT_FN_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_default_uninterpreted_fn(event)
        } else if Self::starts_with(ADD_FN_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_uninterpreted_fn(event)
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let fn_id_str = at_path.first().unwrap();
            let fn_id = self.get_uninterpreted_fn_id(fn_id_str)?;

            self.event_modify_uninterpreted_fn(event, &at_path[1..], fn_id)
        }
    }

    /// Perform event of adding a new `uninterpreted fn` component to this `ModelState`.
    /// This variant assumes that ID, arity (and so on) were already given.
    ///
    /// For now, it is assumed that new functions have no constraints (monotonicity, essentiality,
    /// or expression).
    pub(super) fn event_add_uninterpreted_fn(
        &mut self,
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/uninterpreted_fn";

        // parse the payload
        let payload = Self::clone_payload_str(event, component_name)?;
        let fn_data = UninterpretedFnData::from_json_str(payload.as_str())?;
        let arity = fn_data.arguments.len();
        // add funtion in two steps (to also include annotation)
        self.add_empty_uninterpreted_fn_by_str(&fn_data.id, &fn_data.name, arity)?;
        self.set_fn_annot_by_str(&fn_data.id, &fn_data.annotation)?;

        // prepare the state-change and reverse event (which is a remove event)
        let reverse_at_path = ["uninterpreted_fn", &fn_data.id, "remove"];
        let reverse_event = mk_model_event(&reverse_at_path, None);
        Ok(make_reversible(event.clone(), event, reverse_event))
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
        let uninterpreted_fn = UninterpretedFn::new_without_constraints(fn_id.as_str(), arity)?;
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
            // check that payload is really empty
            if event.payload.is_some() {
                let message = "Payload must be empty for uninterpreted fn removing.".to_string();
                return AeonError::throw(message);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            // note that to remove an uninterpreted_fn, it must not be used in any update fn (checked during the call)
            let fn_data = UninterpretedFnData::from_fn(&fn_id, self.get_uninterpreted_fn(&fn_id)?);
            self.remove_uninterpreted_fn(&fn_id)?;
            let state_change = mk_model_state_change(&["uninterpreted_fn", "remove"], &fn_data);

            // prepare the reverse event
            let reverse_at_path = ["uninterpreted_fn", "add"];
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

            // perform the ID change (which can modify many parts of the model)
            self.set_uninterpreted_fn_id_by_str(fn_id.as_str(), new_id.as_str())?;

            // This event is a bit special - since variable ID change can affect many parts of the model
            // (update fns, regulations, layout, ...), the event returns the whole updated model data to the FE.
            let model_data = ModelData::from_model(self);
            let state_change = mk_model_state_change(&["uninterpreted_fn", "set_id"], &model_data);

            // prepare the reverse event
            let reverse_at_path = ["uninterpreted_fn", new_id.as_str(), "set_id"];
            let reverse_event = mk_model_event(&reverse_at_path, Some(fn_id.as_str()));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_ARITY_PATH, at_path).is_some() {
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
