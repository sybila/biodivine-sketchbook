use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::{AeonError, DynError};
use crate::sketchbook::layout::{LayoutId, NodePosition};
use crate::sketchbook::simplified_structs::{
    LayoutData, LayoutNodeData, RegulationData, UninterpretedFnData, VariableData,
};
use crate::sketchbook::{Essentiality, ModelState, Monotonicity, UninterpretedFnId, VarId};

use serde_json::json;
use std::str::FromStr;

impl SessionHelper for ModelState {}

/// Functionality and shorthands related to the `perform_event` method of the `SessionState` trait.
impl ModelState {
    /// Perform event of adding a new `variable` component to this `ModelState`.
    fn perform_variable_add_event(&mut self, event: &Event) -> Result<Consumed, DynError> {
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
    fn perform_variable_modify_event(
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
    fn perform_variable_event(
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
            self.perform_variable_add_event(event)
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let var_id_str = at_path.first().unwrap();
            let var_id = self.get_var_id(var_id_str)?;
            self.perform_variable_modify_event(event, &at_path[1..], var_id)
        }
    }

    /// Perform event of adding a new `uninterpreted fn` component to this `ModelState`.
    fn perform_uninterpreted_fn_add_event(&mut self, event: &Event) -> Result<Consumed, DynError> {
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
    fn perform_uninterpreted_fn_modify_event(
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
            let new_arity: u32 = Self::clone_payload_str(event, component_name)?.parse()?;
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
    }

    /// Perform events related to `uninterpreted fns` component of this `ModelState`.
    fn perform_uninterpreted_fn_event(
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
            self.perform_uninterpreted_fn_add_event(event)
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let fn_id_str = at_path.first().unwrap();
            let fn_id = self.get_uninterpreted_fn_id(fn_id_str)?;

            self.perform_uninterpreted_fn_modify_event(event, &at_path[1..], fn_id)
        }
    }

    /// Perform event of adding a new `regulation` component to this `ModelState`.
    fn perform_regulation_add_event(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        // get payload components (json for RegulationData containing "regulator", "target",
        // "sign", "essential")
        let payload = Self::clone_payload_str(event, component_name)?;
        let regulation_data = RegulationData::from_str(payload.as_str())?;
        let regulator_id = self.get_var_id(&regulation_data.regulator)?;
        let target_id = self.get_var_id(&regulation_data.target)?;
        let sign: Monotonicity = regulation_data.sign;
        let essential: Essentiality = regulation_data.essential;

        // perform the event, prepare the state-change variant (path and payload stay the same)
        self.add_regulation(regulator_id, target_id, essential, sign)?;
        let state_change = event.clone();

        // prepare the reverse event (it is a remove event, so IDs are in path and payload is empty)
        let reverse_path = [
            "model",
            "regulation",
            &regulation_data.regulator,
            &regulation_data.target,
            "remove",
        ];
        let reverse_event = Event::build(&reverse_path, None);

        Ok(Consumed::Reversible {
            state_change,
            perform_reverse: (event.clone(), reverse_event),
        })
    }

    /// Perform event of modifying or removing existing `regulation` component of this `ModelState`.
    fn perform_regulation_modify_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
        regulator_id: VarId,
        target_id: VarId,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        if Self::starts_with("remove", at_path).is_some() {
            // check that payload is really empty
            if event.payload.is_some() {
                let message = "Payload must be empty for regulation removing.".to_string();
                return AeonError::throw(message);
            }

            // save the original regulation data for state change and reverse event
            let original_reg = self.get_regulation(&regulator_id, &target_id)?.clone();
            let reg_data = RegulationData::from_reg(&original_reg);

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_regulation(&regulator_id, &target_id)?;
            let state_change_path = ["model", "regulation", "remove"];
            let state_change = Event::build(&state_change_path, Some(&reg_data.to_string()));

            // prepare the reverse 'add' event (path has no ids, all info carried by payload)
            let reverse_path = ["model", "regulation", "add"];
            let reverse_event = Event::build(&reverse_path, Some(&reg_data.to_string()));
            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("set_sign", at_path).is_some() {
            // get the payload - a string for the "new_sign"
            let sign_str = Self::clone_payload_str(event, component_name)?;
            let new_sign: Monotonicity = serde_json::from_str(&sign_str)?;

            let original_reg = self.get_regulation(&regulator_id, &target_id)?;
            let orig_sign = *original_reg.get_sign();

            if orig_sign == new_sign {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.change_regulation_sign(&regulator_id, &target_id, &new_sign)?;
            let new_reg = self.get_regulation(&regulator_id, &target_id)?;
            let reg_data = RegulationData::from_reg(new_reg);
            let state_change_path = ["model", "regulation", "set_sign"];
            let state_change = Event::build(&state_change_path, Some(&reg_data.to_string()));

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(serde_json::to_string(&orig_sign)?);
            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("set_essentiality", at_path).is_some() {
            // get the payload - a string for the "new_essentiality"
            let essentiality_str = Self::clone_payload_str(event, component_name)?;
            let new_essentiality: Essentiality = serde_json::from_str(&essentiality_str)?;
            let original_reg = self.get_regulation(&regulator_id, &target_id)?;
            let orig_essentiality = *original_reg.get_essentiality();
            if orig_essentiality == new_essentiality {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.change_regulation_essentiality(&regulator_id, &target_id, &new_essentiality)?;
            let new_reg = self.get_regulation(&regulator_id, &target_id)?;
            let reg_data = RegulationData::from_reg(new_reg);
            let state_change_path = ["model", "regulation", "set_essentiality"];
            let state_change = Event::build(&state_change_path, Some(&reg_data.to_string()));

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(serde_json::to_string(&orig_essentiality)?);
            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }

    /// Perform events related to `regulations` component of this `ModelState`.
    fn perform_regulation_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        // there is either adding of a new regulation, or editing/removing of an existing one
        // when adding new regulation, the `at_path` is just ["add"]
        // when editing existing variable, the `at_path` is ["regulator", "target", "<action>"]

        if Self::starts_with("add", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.perform_regulation_add_event(event)
        } else {
            Self::assert_path_length(at_path, 3, component_name)?;
            let regulator_id_str = at_path.first().unwrap();
            let target_id_str = at_path.get(1).unwrap();
            let regulator_id = self.get_var_id(regulator_id_str)?;
            let target_id = self.get_var_id(target_id_str)?;

            self.perform_regulation_modify_event(event, &at_path[2..], regulator_id, target_id)
        }
    }

    /// Perform event of adding a new `layout` component to this `ModelState`.
    fn perform_layout_add_event(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/layout";

        // get payload components (json for LayoutData)
        let payload = Self::clone_payload_str(event, component_name)?;
        let layout_data = LayoutData::from_str(payload.as_str())?;
        let layout_id_str = layout_data.id;
        let layout_id = self.generate_layout_id(&layout_id_str);
        let name = layout_data.name;

        // perform the event, prepare the state-change variant (path and payload stay the same)
        // todo: decide on how to add layouts - now we just make a copy of the default layout
        self.add_layout_copy(layout_id, name.as_str(), &Self::get_default_layout_id())?;
        let state_change = event.clone();

        // prepare the reverse event (it is a remove event, so IDs are in path and payload is empty)
        let reverse_path = ["model", "layout", layout_id_str.as_str(), "remove"];
        let reverse_event = Event::build(&reverse_path, None);
        Ok(Consumed::Reversible {
            state_change,
            perform_reverse: (event.clone(), reverse_event),
        })
    }

    /// Perform event of modifying or removing existing `layout` component of this `ModelState`.
    fn perform_layout_modify_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
        layout_id: LayoutId,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/layout";

        if Self::starts_with("update_position", at_path).is_some() {
            // get payload components (json containing "var_id", "new_x" and "new_y")
            let payload = Self::clone_payload_str(event, component_name)?;
            let new_node_data = LayoutNodeData::from_str(payload.as_str())?;
            let var_id = self.get_var_id(new_node_data.variable.as_str())?;
            let new_x = new_node_data.px;
            let new_y = new_node_data.py;
            let new_position = NodePosition(new_x, new_y);

            let orig_pos = self.get_node_position(&layout_id, &var_id)?.clone();
            let orig_pos_data = LayoutNodeData::new(
                layout_id.to_string(),
                var_id.to_string(),
                orig_pos.0,
                orig_pos.1,
            );
            let new_pos_data =
                LayoutNodeData::new(layout_id.to_string(), var_id.to_string(), new_x, new_y);

            if new_position == orig_pos {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move ID from path to payload)
            self.update_node_position(&layout_id, &var_id, new_x, new_y)?;
            let state_change_path = ["model", "layout", "update_position"];
            let state_change = Event::build(&state_change_path, Some(&new_pos_data.to_string()));

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(orig_pos_data.to_string());
            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("remove", at_path).is_some() {
            // check that payload is really empty
            if event.payload.is_some() {
                let message = "Payload must be empty for layout removing.".to_string();
                return AeonError::throw(message);
            }

            let layout = self.get_layout(&layout_id)?;
            let layout_data = LayoutData::from_layout(&layout_id, layout);

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.remove_layout(&layout_id)?;
            let state_change_path = ["model", "layout", "remove"];
            let state_change = Event::build(&state_change_path, Some(&layout_data.to_string()));

            // todo make reversible in the future?
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }

    /// Perform events related to `layouts` component of this `ModelState`.
    fn perform_layout_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/layout";

        // there is either adding of a new layout, or editing/removing of an existing one
        // when adding new layout, the `at_path` is just ["add"]
        // when editing existing layout, the `at_path` is ["layout_id", "<action>"]

        if Self::starts_with("add", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.perform_layout_add_event(event)
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let layout_id_str = at_path.first().unwrap();
            let layout_id = self.get_layout_id(layout_id_str)?;

            self.perform_layout_modify_event(event, &at_path[1..], layout_id)
        }
    }
}

/// Functionality and shorthands related to the `refresh` method of the `SessionState` trait.
impl ModelState {
    /// Get a list of all variables.
    fn refresh_variables(&self, full_path: &[String]) -> Result<Event, DynError> {
        let variable_list: Vec<VariableData> = self
            .variables
            .iter()
            .map(|(id, data)| VariableData::from_var(id, data))
            .collect();

        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(serde_json::to_string(&variable_list)?),
        })
    }

    /// Get a list of all uninterpreted fns.
    fn refresh_uninterpreted_fns(&self, full_path: &[String]) -> Result<Event, DynError> {
        let uninterpreted_fn_list: Vec<UninterpretedFnData> = self
            .uninterpreted_fns
            .iter()
            .map(|(id, data)| UninterpretedFnData::from_uninterpreted_fn(id, data))
            .collect();

        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(serde_json::to_string(&uninterpreted_fn_list)?),
        })
    }

    /// Get a list of all regulations.
    fn refresh_regulations(&self, full_path: &[String]) -> Result<Event, DynError> {
        let regulation_list: Vec<RegulationData> = self
            .regulations
            .iter()
            .map(RegulationData::from_reg)
            .collect();

        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(serde_json::to_string(&regulation_list)?),
        })
    }

    /// Get a list of all layouts (just basic information like IDs and names).
    fn refresh_layouts(&self, full_path: &[String]) -> Result<Event, DynError> {
        let layout_list: Vec<LayoutData> = self
            .layouts
            .iter()
            .map(|(id, layout)| LayoutData::from_layout(id, layout))
            .collect();

        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(serde_json::to_string(&layout_list)?),
        })
    }

    /// Get a list with all nodes in a specified layout.
    fn refresh_layout_nodes(
        &self,
        full_path: &[String],
        at_path: &[&str],
    ) -> Result<Event, DynError> {
        Self::assert_path_length(at_path, 1, "model/layout_nodes")?;
        let layout_id_str = at_path.first().unwrap();
        let layout_id = self.get_layout_id(layout_id_str)?;
        let layout = self.get_layout(&layout_id)?;

        let node_list: Vec<LayoutNodeData> = layout
            .layout_nodes()
            .map(|(var_id, node)| {
                LayoutNodeData::from_node(layout_id.to_string(), var_id.to_string(), node)
            })
            .collect();

        // remove the id from the path
        let mut result_path = full_path.to_vec();
        result_path.pop();

        Ok(Event {
            path: result_path,
            payload: Some(serde_json::to_string(&node_list)?),
        })
    }
}

impl SessionState for ModelState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        match at_path.first() {
            Some(&"variable") => self.perform_variable_event(event, &at_path[1..]),
            Some(&"uninterpreted_fn") => self.perform_uninterpreted_fn_event(event, &at_path[1..]),
            Some(&"regulation") => self.perform_regulation_event(event, &at_path[1..]),
            Some(&"layout") => self.perform_layout_event(event, &at_path[1..]),
            _ => Self::invalid_path_error_generic(at_path),
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        match at_path.first() {
            Some(&"get_variables") => self.refresh_variables(full_path),
            Some(&"get_uninterpreted_fns") => self.refresh_uninterpreted_fns(full_path),
            Some(&"get_regulations") => self.refresh_regulations(full_path),
            Some(&"get_layouts") => self.refresh_layouts(full_path),
            Some(&"get_layout_nodes") => self.refresh_layout_nodes(full_path, &at_path[1..]),
            _ => Self::invalid_path_error_generic(at_path),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::event::Event;
    use crate::app::state::{Consumed, SessionState};
    use crate::sketchbook::layout::NodePosition;
    use crate::sketchbook::simplified_structs::{
        LayoutData, LayoutNodeData, RegulationData, VariableData,
    };
    use crate::sketchbook::{Essentiality, ModelState, Monotonicity, VarId};
    use serde_json::json;

    /// Check that after applying the reverse event of `result` to the `model` with relative
    /// path `at_path`, we receive precisely `model_orig`.
    fn check_reverse(
        mut model: ModelState,
        model_orig: ModelState,
        result: Consumed,
        at_path: &[&str],
    ) {
        // assert that the reverse event is correct
        match result {
            Consumed::Reversible {
                perform_reverse: (_, reverse),
                ..
            } => {
                model.perform_event(&reverse, &at_path).unwrap();
                assert_eq!(model, model_orig);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_add_var_event() {
        let mut model = ModelState::new();
        let var_id_a = model.generate_var_id("a");
        model.add_var(var_id_a, "a").unwrap();
        let model_orig = model.clone();
        assert_eq!(model.num_vars(), 1);

        // test variable add event
        let var_data = VariableData::new("b", "b");
        let payload = serde_json::to_string(&var_data).unwrap();
        let full_path = ["model", "variable", "add"];
        let event = Event::build(&full_path, Some(payload.as_str()));
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        // check var was added
        assert_eq!(model.num_vars(), 2);
        assert_eq!(model.get_var_id("b").unwrap(), VarId::new("b").unwrap());
        check_reverse(model, model_orig, result, &["variable", "b", "remove"]);
    }

    #[test]
    /// Test removing a variable that has no regulations and its node is in default position.
    fn test_remove_var_event_simple() {
        let variables = vec![("a", "a_name"), ("b", "b_name")];
        let mut model = ModelState::new_from_vars(variables).unwrap();
        let model_orig = model.clone();

        // test variable remove event
        let full_path = ["model", "variable", "a", "remove"];
        let event = Event::build(&full_path, None);
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        // check var was removed - result should be a simple `Consumed::Reversible` object
        assert_eq!(model.num_vars(), 1);
        check_reverse(model, model_orig, result, &["variable", "add"]);
    }

    #[test]
    /// Test removing a variable that has regulations and its node is in non-default position.
    fn test_remove_var_event_complex() {
        let mut model = ModelState::new();
        let var_id_a = model.generate_var_id("a");
        model.add_var(var_id_a.clone(), "a").unwrap();
        model.add_var_by_str("b", "b").unwrap();
        model
            .add_multiple_regulations(vec!["a -> a", "a -> b", "b -> b"])
            .unwrap();
        model
            .update_node_position(&ModelState::get_default_layout_id(), &var_id_a, 1., 1.)
            .unwrap();

        // expected result
        let mut model_expected = ModelState::new();
        model_expected.add_var_by_str("b", "b").unwrap();
        model_expected.add_regulation_by_str("b -> b").unwrap();

        // test variable remove event
        let full_path = ["model", "variable", "a", "remove"];
        let event = Event::build(&full_path, None);
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        // result should be a `Consumed::Restart` object with a vector of events which should simulate the individual
        // steps of the variable's removal
        if let Consumed::Restart(mut sub_events) = result {
            // there should be 2 events for regulations removal, 1 for re-position, and final one for var removal
            assert_eq!(sub_events.len(), 4);
            sub_events.reverse();
            for e in sub_events {
                let mut full_path = e.path.clone();
                full_path.remove(0);
                let full_path_str: Vec<&str> = full_path.iter().map(|s| s.as_str()).collect();
                println!("{:?}", e);
                model.perform_event(&e, &full_path_str).unwrap();
            }
            assert_eq!(model, model_expected);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_set_var_name_event() {
        let mut model = ModelState::new();
        let var_id = model.generate_var_id("a");
        let original_name = "a_name";
        model.add_var(var_id.clone(), original_name).unwrap();
        let new_name = "new_name";
        let model_orig = model.clone();
        assert_eq!(model.get_var_name(&var_id).unwrap(), original_name);

        // test variable rename event
        let full_path = ["model", "variable", var_id.as_str(), "set_name"];
        let event = Event::build(&full_path, Some(new_name));
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        // check var was renamed
        assert_eq!(model.get_var_name(&var_id).unwrap(), new_name);
        check_reverse(model, model_orig, result, &full_path[1..]);
    }

    #[test]
    fn test_set_var_id_event() {
        let mut model = ModelState::new();
        let var_id = model.generate_var_id("a");
        model.add_var(var_id.clone(), "a_name").unwrap();
        let model_orig = model.clone();

        // test id change event
        let new_id = model.generate_var_id("b");
        let full_path = ["model", "variable", var_id.as_str(), "set_id"];
        let event = Event::build(&full_path, Some(new_id.as_str()));
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        // check id changed
        assert!(!model.is_valid_var_id(&var_id));
        assert!(model.is_valid_var_id(&new_id));
        check_reverse(
            model,
            model_orig,
            result,
            &["variable", new_id.as_str(), "set_id"],
        );
    }

    #[test]
    fn test_invalid_var_events() {
        let mut model = ModelState::new();
        let var_id = model.generate_var_id("a");
        model.add_var(var_id.clone(), "a-name").unwrap();
        let model_orig = model.clone();

        // adding variable `a` again
        let full_path = ["model", "variable", "add"];
        let event = Event::build(&full_path, Some("a"));
        assert!(model.perform_event(&event, &full_path[1..]).is_err());
        assert_eq!(model, model_orig);

        // removing variable with wrong id
        let full_path = ["model", "variable", "b", "remove"];
        let event = Event::build(&full_path, None);
        assert!(model.perform_event(&event, &full_path[1..]).is_err());
        assert_eq!(model, model_orig);

        // variable rename event with wrong id
        let full_path = ["model", "variable", "b", "set_name"];
        let event = Event::build(&full_path, Some("new_name"));
        assert!(model.perform_event(&event, &full_path[1..]).is_err());
        assert_eq!(model, model_orig);
    }

    #[test]
    fn test_add_reg_event() {
        let variables = vec![("a", "a"), ("b", "b")];
        let mut model = ModelState::new_from_vars(variables).unwrap();
        let model_orig = model.clone();

        // test regulation add event
        let full_path = ["model", "regulation", "add"];
        let regulation_data = RegulationData::try_from_reg_str("a -> b").unwrap();
        let event = Event::build(&full_path, Some(&regulation_data.to_string()));
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        assert_eq!(model.num_regulations(), 1);
        check_reverse(
            model,
            model_orig,
            result,
            &["regulation", "a", "b", "remove"],
        );
    }

    #[test]
    fn test_change_reg_sign_event() {
        let variables = vec![("a", "a_name"), ("b", "b_name")];
        let mut model = ModelState::new_from_vars(variables).unwrap();
        let regulations = vec!["a -> a", "a -> b", "b -> a"];
        model.add_multiple_regulations(regulations).unwrap();
        let model_orig = model.clone();

        // test event for changing regulation's sign
        let full_path = ["model", "regulation", "a", "b", "set_sign"];
        let new_sign = serde_json::to_string(&Monotonicity::Inhibition).unwrap();
        let event = Event::build(&full_path, Some(&new_sign));
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        check_reverse(
            model,
            model_orig,
            result,
            &["regulation", "a", "b", "set_sign"],
        );
    }

    #[test]
    fn test_change_reg_essentiality_event() {
        let variables = vec![("a", "a_name"), ("b", "b_name")];
        let mut model = ModelState::new_from_vars(variables).unwrap();
        let regulations = vec!["a -> a", "a -> b", "b -> a"];
        model.add_multiple_regulations(regulations).unwrap();
        let model_orig = model.clone();

        // test event for changing regulation's essentiality
        let full_path = ["model", "regulation", "a", "b", "set_essentiality"];
        let new_essentiality = serde_json::to_string(&Essentiality::False).unwrap();
        let event = Event::build(&full_path, Some(&new_essentiality));
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        check_reverse(
            model,
            model_orig,
            result,
            &["regulation", "a", "b", "set_essentiality"],
        );
    }

    #[test]
    fn test_remove_reg_event() {
        let variables = vec![("a", "a_name"), ("b", "b_name")];
        let mut model = ModelState::new_from_vars(variables).unwrap();
        model.add_regulation_by_str("a -> b").unwrap();
        let model_orig = model.clone();

        // test regulation add event
        let full_path = ["model", "regulation", "a", "b", "remove"];
        let event = Event::build(&full_path, None);
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        assert_eq!(model.num_regulations(), 0);
        check_reverse(model, model_orig, result, &["regulation", "add"]);
    }

    #[test]
    fn test_change_position_event() {
        let mut model = ModelState::new();
        let layout_id = ModelState::get_default_layout_id();
        let var_id = model.generate_var_id("a");
        model.add_var(var_id.clone(), "a_name").unwrap();
        let model_orig = model.clone();

        // test position change event
        let payload = json!({
            "layout": layout_id.as_str(),
            "variable": var_id.as_str(),
            "px": 2.5,
            "py": 0.4,
        })
        .to_string();
        let full_path = ["model", "layout", layout_id.as_str(), "update_position"];
        let event = Event::build(&full_path, Some(payload.as_str()));
        let result = model.perform_event(&event, &full_path[1..]).unwrap();

        // check position changed
        assert_eq!(
            model.get_node_position(&layout_id, &var_id).unwrap(),
            &NodePosition(2.5, 0.4)
        );

        check_reverse(model, model_orig, result, &full_path[1..]);
    }

    #[test]
    fn test_refresh() {
        let mut model = ModelState::new();
        let layout_id = ModelState::get_default_layout_id().to_string();
        let var_id = model.generate_var_id("a");
        model.add_var(var_id.clone(), "a_name").unwrap();
        model.add_regulation_by_str("a -> a").unwrap();

        // test variable getter
        let event = model
            .refresh(
                &["model".to_string(), "get_variables".to_string()],
                &["get_variables"],
            )
            .unwrap();
        let var_list: Vec<VariableData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
        assert_eq!(var_list.len(), 1);

        // test regulation getter
        let event = model
            .refresh(
                &["model".to_string(), "get_regulations".to_string()],
                &["get_regulations"],
            )
            .unwrap();
        let reg_list: Vec<RegulationData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
        assert_eq!(reg_list.len(), 1);

        // test layout getter
        let event = model
            .refresh(
                &["model".to_string(), "get_layouts".to_string()],
                &["get_layouts"],
            )
            .unwrap();
        let layout_list: Vec<LayoutData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
        assert_eq!(layout_list.first().unwrap().id, layout_id.clone());

        // test layout node getter
        let event = model
            .refresh(
                &[
                    "model".to_string(),
                    "get_layout_nodes".to_string(),
                    layout_id.clone(),
                ],
                &["get_layout_nodes", &layout_id],
            )
            .unwrap();
        let node_list: Vec<LayoutNodeData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
        assert_eq!(node_list.first().unwrap().variable, var_id.to_string());
    }
}
