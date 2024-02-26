use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::{AeonError, DynError};
use crate::sketchbook::layout::NodePosition;
use crate::sketchbook::simplified_structs::{LayoutNodeData, VariableData};
use crate::sketchbook::{ModelState, VarId};

use serde_json::json;
use std::str::FromStr;

/// Implementation for events related to `variables` of the model.
impl ModelState {
    /// Perform event of adding a new `variable` component to this `ModelState`.
    pub(super) fn event_add_variable(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/variable";

        // get payload components (json for VariableData containing "id", and "name")
        let payload = Self::clone_payload_str(event, component_name)?;
        let variable_data = VariableData::from_str(payload.as_str())?;
        let var_id_str = variable_data.id;
        let name = variable_data.name;

        // perform the event, prepare the state-change variant (path and payload stay the same)
        self.add_var_by_str(var_id_str.as_str(), name.as_str())?;
        let state_change = event.clone();

        // prepare the reverse event (it is a remove event, so ID is in path and payload is empty)
        let reverse_path = ["model", "variable", var_id_str.as_str(), "remove"];
        let reverse_event = Event::build(&reverse_path, None);
        Ok(Consumed::Reversible {
            state_change,
            perform_reverse: (event.clone(), reverse_event),
        })
    }

    /// Perform event of modifying or removing existing `variable` component of this `ModelState`.
    pub(super) fn event_modify_variable(
        &mut self,
        event: &Event,
        at_path: &[&str],
        var_id: VarId,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/variable";

        if Self::starts_with("remove", at_path).is_some() {
            // check that payload is really empty
            if event.payload.is_some() {
                let message = "Payload must be empty for variable removing.".to_string();
                return AeonError::throw(message);
            }

            // To remove a variable, all its regulations must be already removed, and it must be at default position
            // in each layout. If it is not the case, to ensure that we can undo this operation, we precede the var
            // removal with a set of events to remove all its regulations, and move its nodes to default positions
            // (as separate undo-able events).

            let targets = self.targets(&var_id)?;
            let regulators = self.regulators(&var_id)?;
            let needs_to_move = self.layouts.iter().fold(true, |acc, (_, l)| {
                acc && (l.get_node_position(&var_id).unwrap() != &NodePosition(0., 0.))
            });

            if regulators.is_empty() && targets.is_empty() && !needs_to_move {
                // perform the event, prepare the state-change variant (move id from path to payload)
                let var_data = VariableData::from_var(&var_id, self.get_variable(&var_id)?);
                self.remove_var(&var_id)?;
                let state_change_path = ["model", "variable", "remove"];
                let state_change = Event::build(&state_change_path, Some(&var_data.to_string()));

                // prepare the reverse event
                let reverse_path = ["model", "variable", "add"];
                let reverse_event = Event::build(&reverse_path, Some(&var_data.to_string()));
                Ok(Consumed::Reversible {
                    state_change,
                    perform_reverse: (event.clone(), reverse_event),
                })
            } else {
                let mut event_list = Vec::new();
                event_list.push(event.clone());
                for l_id in self.layouts.keys() {
                    let event_path = ["model", "layout", l_id.as_str(), "update_position"];
                    let payload = LayoutNodeData::new(l_id.to_string(), var_id.to_string(), 0., 0.)
                        .to_string();
                    let move_event = Event::build(&event_path, Some(payload.as_str()));
                    event_list.push(move_event)
                }
                for reg in regulators {
                    let event_path = [
                        "model",
                        "regulation",
                        reg.as_str(),
                        var_id.as_str(),
                        "remove",
                    ];
                    let remove_event = Event::build(&event_path, None);
                    event_list.push(remove_event)
                }
                for target in targets {
                    // if both target and regulator is the same var (self-loop), it was already removed
                    if var_id == *target {
                        continue;
                    }
                    let event_path = [
                        "model",
                        "regulation",
                        var_id.as_str(),
                        target.as_str(),
                        "remove",
                    ];
                    let remove_event = Event::build(&event_path, None);
                    event_list.push(remove_event)
                }
                Ok(Consumed::Restart(event_list))
            }
        } else if Self::starts_with("set_name", at_path).is_some() {
            // get the payload - string for "new_name"
            let new_name = Self::clone_payload_str(event, component_name)?;
            let original_name = self.get_var_name(&var_id)?.to_string();
            if new_name == original_name {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_var_name(&var_id, new_name.as_str())?;
            let var_data = VariableData::from_var(&var_id, self.get_variable(&var_id)?);
            let state_change_path = ["model", "variable", "set_name"];
            let state_change = Event::build(&state_change_path, Some(&var_data.to_string()));

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
            let new_var_id = self.generate_var_id(new_id.as_str());
            if var_id == new_var_id {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_var_id(&var_id, new_var_id)?;
            let state_change_path = ["model", "variable", "set_id"];
            let payload = json!({
                "original_id": var_id.as_str(),
                "new_id": new_id.as_str(),
            });
            let state_change = Event::build(&state_change_path, Some(&payload.to_string()));

            // prepare the reverse event
            let reverse_event_path = ["model", "variable", new_id.as_str(), "set_id"];
            let reverse_event = Event::build(&reverse_event_path, Some(var_id.as_str()));
            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }

    /// Perform events related to `variables` component of this `ModelState`.
    pub(super) fn perform_variable_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/variable";

        // there is either adding of a new variable, or editing/removing of an existing one
        // when adding new variable, the `at_path` is just ["add"]
        // when editing existing variable, the `at_path` is ["var_id", "<action>"]

        if Self::starts_with("add", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_variable(event)
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let var_id_str = at_path.first().unwrap();
            let var_id = self.get_var_id(var_id_str)?;
            self.event_modify_variable(event, &at_path[1..], var_id)
        }
    }
}
