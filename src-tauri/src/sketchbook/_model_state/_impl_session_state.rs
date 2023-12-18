use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::{AeonError, DynError};
use crate::sketchbook::layout::{LayoutId, NodePosition};
use crate::sketchbook::simplified_structs::{
    LayoutData, LayoutNodeData, RegulationData, VariableData,
};
use crate::sketchbook::{ModelState, Observability, RegulationSign, VarId};

use serde_json::json;

impl SessionHelper for ModelState {}

/// Functionality and shorthands related for the `SessionState` trait.
impl ModelState {
    /// Shorthand to get and clone a payload of an event. Errors if payload is empty.
    /// The `component` specifies which part of the state should be mentioned in the error.
    /// In future we may consider moving this elsewhere.
    fn clone_payload_str(event: &Event, component: &str) -> Result<String, DynError> {
        let payload = event.payload.clone().ok_or(format!(
            "Event to `{component}` cannot carry empty payload."
        ))?;
        Ok(payload)
    }

    /// Shorthand to assert that path has given length and return typesafe `DynError` otherwise.
    /// The `component` specifies which component of the state should be mentioned in the error.
    fn assert_path_length(path: &[&str], length: usize, component: &str) -> Result<(), DynError> {
        if path.len() != length {
            return AeonError::throw(format!("`{component}` cannot consume a path `{:?}`.", path));
        }
        Ok(())
    }

    fn throw_path_error<T>(path: &[&str], component: &str) -> Result<T, DynError> {
        AeonError::throw(format!("`{component}` cannot consume a path `{:?}`.", path))
    }

    /// Perform event of adding a new `variable` component to this `ModelState`.
    fn perform_variable_add_event(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/variable";

        // get payload components (json for VariableData containing "id", and "name")
        let payload = Self::clone_payload_str(event, component_name)?;
        let variable_data: VariableData = serde_json::from_str(payload.as_str())?;
        let var_id_str = variable_data.id;
        let name = variable_data.name;

        // perform the event, prepare the state-change variant (path and payload stay the same)
        self.add_var_by_str(var_id_str.as_str(), name.as_str())?;
        let state_change = event.clone();

        // prepare the reverse event (it is a remove event, so IDs are in path and payload is empty)
        let mut reverse_path = event.path.clone();
        reverse_path.remove(reverse_path.len() - 1);
        reverse_path.push(var_id_str.to_string());
        reverse_path.push("remove".to_string());
        let reverse_path: Vec<&str> = reverse_path.iter().map(|s| s.as_str()).collect();
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

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.remove_var(&var_id)?;
            let state_change_path = ["model", "variable", "remove"];
            let state_change = Event::build(&state_change_path, Some(var_id.as_str()));

            // todo make reversible in the future
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with("set_name", at_path).is_some() {
            // get the payload - string for "new_name"
            let new_name = Self::clone_payload_str(event, component_name)?;
            let original_name = self.get_var_name(&var_id)?.to_string();

            if new_name == original_name {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_var_name(&var_id, new_name.as_str())?;
            let state_change_path = ["model", "variable", "set_name"];
            let state_change_payload = json!({
                "id": var_id.as_str(),
                "name": new_name,
            });
            let state_change =
                Event::build(&state_change_path, Some(&state_change_payload.to_string()));

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
            let state_change_payload = json!({
                "original_id": var_id.as_str(),
                "new_id": new_id.as_str(),
            });
            let state_change =
                Event::build(&state_change_path, Some(&state_change_payload.to_string()));

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(var_id.to_string());
            let path_len = reverse_event.path.len();
            let var_id_path = reverse_event.path.get_mut(path_len - 2).unwrap();
            *var_id_path = new_id.to_string();

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else {
            Self::throw_path_error(at_path, component_name)
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
        // when editing existing variable, the `at_path` is ["var_id", "action"]

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

    /// Perform event of adding a new `regulation` component to this `ModelState`.
    fn perform_regulation_add_event(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        // get payload components (json for RegulationData containing "regulator", "target",
        // "sign", "observable")
        let payload = Self::clone_payload_str(event, component_name)?;
        let regulation_data: RegulationData = serde_json::from_str(payload.as_str())?;
        let regulator_id = self.get_var_id(&regulation_data.regulator)?;
        let target_id = self.get_var_id(&regulation_data.target)?;
        let sign: RegulationSign = regulation_data.sign;
        let observable: Observability = regulation_data.observable;

        // perform the event, prepare the state-change variant (path and payload stay the same)
        self.add_regulation(regulator_id, target_id, observable, sign)?;
        let state_change = event.clone();

        // prepare the reverse event (it is a remove event, so IDs are in path and payload is empty)
        let mut reverse_path = event.path.clone();
        reverse_path.remove(reverse_path.len() - 1);
        reverse_path.push(regulation_data.regulator);
        reverse_path.push(regulation_data.target);
        reverse_path.push("remove".to_string());
        let reverse_path: Vec<&str> = reverse_path.iter().map(|s| s.as_str()).collect();
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

            // save the original regulation data for reverse event
            let original_reg = self.get_regulation(&regulator_id, &target_id)?.clone();
            let original_reg_data = RegulationData::from_reg(&original_reg);

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_regulation(&regulator_id, &target_id)?;
            let state_change_path = ["model", "regulation", "remove"];
            let state_change_payload = json!({
                "regulator": regulator_id.as_str(),
                "target": target_id.as_str(),
            });
            let state_change =
                Event::build(&state_change_path, Some(&state_change_payload.to_string()));

            // prepare the reverse event (path has no ids, instead all info carried by payload)
            let reverse_path = ["model", "regulation", "add"];
            let reverse_payload = serde_json::to_string(&original_reg_data)?;
            let reverse_event = Event::build(&reverse_path, Some(&reverse_payload));

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("set_sign", at_path).is_some() {
            // get the payload - a string for the "new_sign"
            let sign_str = Self::clone_payload_str(event, component_name)?;
            let new_sign: RegulationSign = serde_json::from_str(&sign_str)?;

            let orig_sign = *self.get_regulation(&regulator_id, &target_id)?.get_sign();

            if orig_sign == new_sign {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.change_regulation_sign(&regulator_id, &target_id, &new_sign)?;
            let state_change_path = ["model", "regulation", "set_sign"];
            let state_change_payload = json!({
                "regulator": regulator_id.as_str(),
                "target": target_id.as_str(),
                "new_sign": sign_str,
            });
            let state_change =
                Event::build(&state_change_path, Some(&state_change_payload.to_string()));

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(serde_json::to_string(&orig_sign)?);

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else {
            Self::throw_path_error(at_path, component_name)
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
        // when editing existing variable, the `at_path` is ["regulator", "target", "action"]

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

        // get payload components (json for LayoutData containing "id", and "name")
        let payload = Self::clone_payload_str(event, component_name)?;
        let layout_data: LayoutData = serde_json::from_str(payload.as_str())?;
        let layout_id_str = layout_data.id;
        let layout_id = self.generate_layout_id(&layout_id_str);
        let name = layout_data.name;

        // perform the event, prepare the state-change variant (path and payload stay the same)
        // todo: decide on a method how to add layouts - now it is just a copy of the first layout
        self.add_layout_copy(layout_id, name.as_str(), &Self::get_default_layout_id())?;
        let state_change = event.clone();

        // prepare the reverse event (it is a remove event, so IDs are in path and payload is empty)
        let mut reverse_path = event.path.clone();
        reverse_path.remove(reverse_path.len() - 1);
        reverse_path.push(layout_id_str.to_string());
        reverse_path.push("remove".to_string());
        let reverse_path: Vec<&str> = reverse_path.iter().map(|s| s.as_str()).collect();
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
            let new_node_data: LayoutNodeData = serde_json::from_str(payload.as_str())?;
            let var_id = self.get_var_id(new_node_data.var_id.as_str())?;
            let new_x = new_node_data.px;
            let new_y = new_node_data.py;
            let new_position = NodePosition(new_x, new_y);

            let orig_pos = self.get_node_position(&layout_id, &var_id)?.clone();
            let orig_pos_data = LayoutNodeData::new(var_id.to_string(), orig_pos.0, orig_pos.1);

            if new_position == orig_pos {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move ID from path to payload)
            self.update_node_position(&layout_id, &var_id, new_x, new_y)?;
            let state_change_path = ["model", "layout", "update_position"];
            let state_change_payload = json!({
                "layout_id": layout_id.as_str(),
                "var_id": var_id.as_str(),
                "px": new_x,
                "py": new_y,
            });
            let state_change =
                Event::build(&state_change_path, Some(&state_change_payload.to_string()));

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(serde_json::to_string(&orig_pos_data)?);

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

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.remove_layout(&layout_id)?;
            let state_change_path = ["model", "layout", "remove"];
            let state_change = Event::build(&state_change_path, Some(layout_id.as_str()));

            // todo make reversible in the future
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else {
            Self::throw_path_error(at_path, component_name)
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
        // when editing existing layout, the `at_path` is ["layout_id", "action"]

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

impl SessionState for ModelState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        match at_path.first() {
            Some(&"variable") => self.perform_variable_event(event, &at_path[1..]),
            Some(&"regulation") => self.perform_regulation_event(event, &at_path[1..]),
            Some(&"layout") => self.perform_layout_event(event, &at_path[1..]),
            _ => Self::invalid_path_error(at_path),
        }
    }

    /// TODO: change this to make it a valid getter event API
    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        if at_path.is_empty() {
            return Self::invalid_path_error(at_path);
        }
        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(self.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::app::event::Event;
    use crate::app::state::{Consumed, SessionState};
    use crate::sketchbook::layout::NodePosition;
    use crate::sketchbook::simplified_structs::{RegulationData, VariableData};
    use crate::sketchbook::{ModelState, VarId};
    use serde_json::json;

    /// Check that after applying the reverse event of `result` to the `reg_state` with relative
    /// path `at_path`, we receive precisely `reg_state_orig`.
    fn check_reverse(
        mut reg_state: ModelState,
        reg_state_orig: ModelState,
        result: Consumed,
        at_path: &[&str],
    ) {
        // assert that the reverse event is correct
        match result {
            Consumed::Reversible {
                perform_reverse: (_, reverse),
                ..
            } => {
                reg_state.perform_event(&reverse, &at_path).unwrap();
                assert_eq!(reg_state, reg_state_orig);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_add_var_event() {
        let mut reg_state = ModelState::new();
        let var_id_a = reg_state.generate_var_id("a");
        reg_state.add_var(var_id_a, "a").unwrap();
        let reg_state_orig = reg_state.clone();
        assert_eq!(reg_state.num_vars(), 1);

        // test variable add event
        let var_data = VariableData::new("b".to_string(), "b".to_string());
        let payload = serde_json::to_string(&var_data).unwrap();
        let full_path = ["model", "variable", "add"];
        let event = Event::build(&full_path, Some(payload.as_str()));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        // check var was added
        assert_eq!(reg_state.num_vars(), 2);
        assert_eq!(reg_state.get_var_id("b").unwrap(), VarId::new("b").unwrap());
        check_reverse(
            reg_state,
            reg_state_orig,
            result,
            &["variable", "b", "remove"],
        );
    }

    #[test]
    fn test_remove_var_event() {
        let mut reg_state = ModelState::new();
        let var_id_a = reg_state.generate_var_id("a");
        reg_state.add_var(var_id_a, "a").unwrap();
        reg_state.add_regulation_by_str("a -> a").unwrap();
        assert_eq!(reg_state.num_vars(), 1);
        assert_eq!(reg_state.num_regulations(), 1);

        // test variable remove event
        let full_path = ["model", "variable", "a", "remove"];
        let event = Event::build(&full_path, None);
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        // check var was removed
        assert_eq!(reg_state.num_vars(), 0);
        assert_eq!(reg_state.num_regulations(), 0);

        // assert that it is irreversible (at the moment)
        assert!(matches!(result, Consumed::Irreversible { .. }));
    }

    #[test]
    fn test_set_var_name_event() {
        let mut reg_state = ModelState::new();
        let var_id = reg_state.generate_var_id("a");
        let original_name = "a_name";
        reg_state.add_var(var_id.clone(), original_name).unwrap();
        let new_name = "new_name";
        let reg_state_orig = reg_state.clone();
        assert_eq!(reg_state.get_var_name(&var_id).unwrap(), original_name);

        // test variable rename event
        let full_path = ["model", "variable", var_id.as_str(), "set_name"];
        let event = Event::build(&full_path, Some(new_name));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        // check var was renamed
        assert_eq!(reg_state.get_var_name(&var_id).unwrap(), new_name);
        check_reverse(reg_state, reg_state_orig, result, &full_path[1..]);
    }

    #[test]
    fn test_set_var_id_event() {
        let mut reg_state = ModelState::new();
        let var_id = reg_state.generate_var_id("a");
        reg_state.add_var(var_id.clone(), "a_name").unwrap();
        let reg_state_orig = reg_state.clone();

        // test id change event
        let new_id = reg_state.generate_var_id("b");
        let full_path = ["model", "variable", var_id.as_str(), "set_id"];
        let event = Event::build(&full_path, Some(new_id.as_str()));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        // check id changed
        assert!(!reg_state.is_valid_var_id(&var_id));
        assert!(reg_state.is_valid_var_id(&new_id));
        check_reverse(
            reg_state,
            reg_state_orig,
            result,
            &["variable", new_id.as_str(), "set_id"],
        );
    }

    #[test]
    fn test_invalid_var_events() {
        let mut reg_state = ModelState::new();
        let var_id = reg_state.generate_var_id("a");
        reg_state.add_var(var_id.clone(), "a-name").unwrap();
        let reg_state_orig = reg_state.clone();

        // adding variable `a` again
        let full_path = ["model", "variable", "add"];
        let event = Event::build(&full_path, Some("a"));
        assert!(reg_state.perform_event(&event, &full_path[1..]).is_err());
        assert_eq!(reg_state, reg_state_orig);

        // removing variable with wrong id
        let full_path = ["model", "variable", "b", "remove"];
        let event = Event::build(&full_path, None);
        assert!(reg_state.perform_event(&event, &full_path[1..]).is_err());
        assert_eq!(reg_state, reg_state_orig);

        // variable rename event with wrong id
        let full_path = ["model", "variable", "b", "set_name"];
        let event = Event::build(&full_path, Some("new_name"));
        assert!(reg_state.perform_event(&event, &full_path[1..]).is_err());
        assert_eq!(reg_state, reg_state_orig);
    }

    #[test]
    fn test_add_reg_event() {
        let mut reg_state = ModelState::new();
        let var_id_a = reg_state.generate_var_id("a");
        reg_state.add_var(var_id_a.clone(), "a").unwrap();
        let var_id_b = reg_state.generate_var_id("b");
        reg_state.add_var(var_id_b.clone(), "b").unwrap();
        let reg_state_orig = reg_state.clone();

        // test regulation add event
        let full_path = ["model", "regulation", "add"];
        let regulation_data = RegulationData::try_from_reg_str("a -> b").unwrap();
        let event = Event::build(&full_path, Some(&regulation_data.to_string()));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        assert_eq!(reg_state.num_regulations(), 1);
        check_reverse(
            reg_state,
            reg_state_orig,
            result,
            &["regulation", var_id_a.as_str(), var_id_b.as_str(), "remove"],
        );
    }

    #[test]
    fn test_remove_reg_event() {
        let mut reg_state = ModelState::new();
        let var_a = reg_state.generate_var_id("a");
        reg_state.add_var(var_a.clone(), "a").unwrap();
        let varb = reg_state.generate_var_id("b");
        reg_state.add_var(varb.clone(), "b").unwrap();
        reg_state.add_regulation_by_str("a -> b").unwrap();
        let reg_state_orig = reg_state.clone();

        // test regulation add event
        let full_path = [
            "model",
            "regulation",
            var_a.as_str(),
            varb.as_str(),
            "remove",
        ];
        let event = Event::build(&full_path, None);
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        assert_eq!(reg_state.num_regulations(), 0);
        check_reverse(reg_state, reg_state_orig, result, &["regulation", "add"]);
    }

    #[test]
    fn test_change_position_event() {
        let mut reg_state = ModelState::new();
        let layout_id = ModelState::get_default_layout_id();
        let var_id = reg_state.generate_var_id("a");
        reg_state.add_var(var_id.clone(), "a_name").unwrap();
        let reg_state_orig = reg_state.clone();

        // test position change event
        let payload = json!({
            "var_id": var_id.as_str(),
            "px": 2.5,
            "py": 0.4,
        })
        .to_string();
        let full_path = ["model", "layout", layout_id.as_str(), "update_position"];
        let event = Event::build(&full_path, Some(payload.as_str()));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        // check position changed
        assert_eq!(
            reg_state.get_node_position(&layout_id, &var_id).unwrap(),
            &NodePosition(2.5, 0.4)
        );

        check_reverse(reg_state, reg_state_orig, result, &full_path[1..]);
    }
}
