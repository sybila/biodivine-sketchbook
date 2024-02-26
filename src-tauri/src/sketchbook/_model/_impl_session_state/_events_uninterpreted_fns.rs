use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::{AeonError, DynError};
use crate::sketchbook::simplified_structs::UninterpretedFnData;
use crate::sketchbook::{ModelState, UninterpretedFnId};

use serde_json::json;
use std::str::FromStr;

/// Implementation for events related to `uninterpreted functions` of the model.
impl ModelState {
    /// Perform event of adding a new `uninterpreted fn` component to this `ModelState`.
    pub(super) fn event_add_uninterpreted_fn(
        &mut self,
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/uninterpreted_fn";

        let payload = Self::clone_payload_str(event, component_name)?;
        let fn_data = UninterpretedFnData::from_str(payload.as_str())?;
        let fn_id_str = fn_data.id;
        let name = fn_data.name;
        let arity = fn_data.arity;

        // perform the event, prepare the state-change variant (path and payload stay the same)
        self.add_uninterpreted_fn_by_str(fn_id_str.as_str(), name.as_str(), arity)?;
        let state_change = event.clone();

        // prepare the reverse event (it is a remove event, so IDs are in path and payload is empty)
        let reverse_path = ["model", "uninterpreted_fn", fn_id_str.as_str(), "remove"];
        let reverse_event = Event::build(&reverse_path, None);
        Ok(Consumed::Reversible {
            state_change,
            perform_reverse: (event.clone(), reverse_event),
        })
    }

    /// Perform event of modifying or removing existing `uninterpreted fn` component of this `ModelState`.
    pub(super) fn event_modify_uninterpreted_fn(
        &mut self,
        event: &Event,
        at_path: &[&str],
        fn_id: UninterpretedFnId,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/uninterpreted_fn";

        if Self::starts_with("remove", at_path).is_some() {
            // check that payload is really empty
            if event.payload.is_some() {
                let message = "Payload must be empty for uninterpreted fn removing.".to_string();
                return AeonError::throw(message);
            }

            // Note that to remove an uninterpreted_fn, it must not be used in any update fn (checked during the call).

            // perform the event, prepare the state-change variant (move id from path to payload)
            let fn_data = UninterpretedFnData::from_uninterpreted_fn(
                &fn_id,
                self.get_uninterpreted_fn(&fn_id)?,
            );
            self.remove_uninterpreted_fn(&fn_id)?;
            let state_change_path = ["model", "uninterpreted_fn", "remove"];
            let state_change = Event::build(&state_change_path, Some(&fn_data.to_string()));

            // prepare the reverse event
            let reverse_path = ["model", "uninterpreted_fn", "add"];
            let reverse_event = Event::build(&reverse_path, Some(&fn_data.to_string()));
            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("set_name", at_path).is_some() {
            // get the payload - string for "new_name"
            let new_name = Self::clone_payload_str(event, component_name)?;
            let original_name = self.get_uninterpreted_fn(&fn_id)?.get_name().to_string();
            if new_name == original_name {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_uninterpreted_fn_name(&fn_id, new_name.as_str())?;
            let fn_data = UninterpretedFnData::from_uninterpreted_fn(
                &fn_id,
                self.get_uninterpreted_fn(&fn_id)?,
            );
            let state_change_path = ["model", "uninterpreted_fn", "set_name"];
            let state_change = Event::build(&state_change_path, Some(&fn_data.to_string()));

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(original_name);
            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("set_id", at_path).is_some() {
            // get the payload - string for "new_id"
            let new_id = Self::clone_payload_str(event, component_name)?;
            let new_fn_id = self.generate_uninterpreted_fn_id(new_id.as_str());
            if fn_id == new_fn_id {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_uninterpreted_fn_id(&fn_id, new_fn_id)?;
            let state_change_path = ["model", "uninterpreted_fn", "set_id"];
            let payload = json!({
                "original_id": fn_id.as_str(),
                "new_id": new_id.as_str(),
            });
            let state_change = Event::build(&state_change_path, Some(&payload.to_string()));

            // prepare the reverse event
            let reverse_event_path = ["model", "uninterpreted_fn", new_id.as_str(), "set_id"];
            let reverse_event = Event::build(&reverse_event_path, Some(fn_id.as_str()));
            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("set_arity", at_path).is_some() {
            // get the payload - string for "new_name"
            let new_arity: usize = Self::clone_payload_str(event, component_name)?.parse()?;
            let original_arity = self.get_uninterpreted_fn(&fn_id)?.get_arity();
            if new_arity == original_arity {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_uninterpreted_fn_arity(&fn_id, new_arity)?;
            let fn_data = UninterpretedFnData::from_uninterpreted_fn(
                &fn_id,
                self.get_uninterpreted_fn(&fn_id)?,
            );
            let state_change_path = ["model", "uninterpreted_fn", "set_arity"];
            let state_change = Event::build(&state_change_path, Some(&fn_data.to_string()));

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(original_arity.to_string());

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
        // todo: more events - expression or monotonicity/essentiality of arguments
    }

    /// Perform events related to `uninterpreted fns` component of this `ModelState`.
    pub(super) fn perform_uninterpreted_fn_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/uninterpreted_fn";

        // there is either adding of a new uninterpreted_fn, or editing/removing of an existing one
        // when adding new uninterpreted fn, the `at_path` is just ["add"]
        // when editing existing uninterpreted fn, the `at_path` is ["fn_id", "<action>"]

        if Self::starts_with("add", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_uninterpreted_fn(event)
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let fn_id_str = at_path.first().unwrap();
            let fn_id = self.get_uninterpreted_fn_id(fn_id_str)?;

            self.event_modify_uninterpreted_fn(event, &at_path[1..], fn_id)
        }
    }
}
