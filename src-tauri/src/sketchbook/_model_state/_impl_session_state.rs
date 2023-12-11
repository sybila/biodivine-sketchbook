use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::{AeonError, DynError};
use crate::sketchbook::layout::NodePosition;
use crate::sketchbook::ModelState;
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

    /// Shorthand to get a given str field from json object. Errors if no such field or bad format.
    fn str_from_json(json_payload: &serde_json::Value, field_name: &str) -> Result<String, String> {
        let value = json_payload[field_name]
            .as_str()
            .ok_or(format!("Payload {json_payload} is invalid"))?
            .trim_matches('\"')
            .to_string();
        Ok(value)
    }

    /// Shorthand to get a given float from json object. Errors if no such field or bad format.
    fn float_from_json(json_payload: &serde_json::Value, field_name: &str) -> Result<f32, String> {
        let value = json_payload[field_name].as_f64().ok_or(format!(
            "Payload {json_payload} is invalid floating point number"
        ))?;
        Ok(value as f32)
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

    /// Perform events related to `variables` component of this `ModelState`.
    fn perform_variable_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/variable";

        if Self::starts_with("add", at_path).is_some() {
            // get payload components
            let payload = Self::clone_payload_str(event, component_name)?;
            let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
            let var_id_str = Self::str_from_json(&payload_json, "id")?;
            let name = Self::str_from_json(&payload_json, "name")?;

            // perform the event, prepare the state-change variant
            self.add_var_by_str(var_id_str.as_str(), name.as_str())?;
            let state_change = event.clone();

            // prepare the reverse event
            let mut reverse_path = event.path.clone();
            reverse_path.remove(reverse_path.len() - 1);
            reverse_path.push("remove".to_string());
            let reverse_path: Vec<&str> = reverse_path.iter().map(|s| s.as_str()).collect();
            let reverse_event = Event::build(&reverse_path, Some(var_id_str.as_str()));

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("remove", at_path).is_some() {
            let var_id_str = Self::clone_payload_str(event, component_name)?;
            self.remove_var_by_str(var_id_str.as_str())?;
            let state_change = event.clone();
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with("set_name", at_path).is_some() {
            // get payload components
            let payload = Self::clone_payload_str(event, component_name)?;
            let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
            let var_id_str = Self::str_from_json(&payload_json, "id")?;
            let var_id = self.get_var_id(var_id_str.as_str())?;
            let new_name = Self::str_from_json(&payload_json, "new_name")?;
            let original_name = self.get_var_name(&var_id)?.to_string();

            if new_name == original_name {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant
            self.set_var_name(&var_id, new_name.as_str())?;
            let state_change = event.clone();

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(
                json!({
                    "id": var_id.as_str(),
                    "new_name": original_name,
                })
                .to_string(),
            );

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("set_id", at_path).is_some() {
            // get payload components
            let payload = Self::clone_payload_str(event, component_name)?;
            let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
            let original_id = Self::str_from_json(&payload_json, "original_id")?;
            let new_id = Self::str_from_json(&payload_json, "new_id")?;

            if original_id == new_id {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant
            self.set_var_id_by_str(&original_id, &new_id)?;
            let state_change = event.clone();

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(
                json!({
                    "original_id": new_id.as_str(),
                    "new_id": original_id.as_str(),
                })
                .to_string(),
            );

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

        if Self::starts_with("add_by_str", at_path).is_some() {
            // get payload components
            let payload = Self::clone_payload_str(event, component_name)?;

            // perform the event, prepare the state-change variant
            self.add_regulation_by_str(payload.as_str())?;
            let state_change = event.clone();

            // prepare the reverse event
            let mut reverse_path = event.path.clone();
            reverse_path.remove(reverse_path.len() - 1);
            reverse_path.push("remove_by_str".to_string());
            let reverse_path: Vec<&str> = reverse_path.iter().map(|s| s.as_str()).collect();
            let reverse_event = Event::build(&reverse_path, Some(payload.as_str()));

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("remove_by_str", at_path).is_some() {
            // get payload components
            let payload = Self::clone_payload_str(event, component_name)?;

            // perform the event, prepare the state-change variant
            self.remove_regulation_by_str(payload.as_str())?;
            let state_change = event.clone();

            // prepare the reverse event
            let mut reverse_path = event.path.clone();
            reverse_path.remove(reverse_path.len() - 1);
            reverse_path.push("add_by_str".to_string());
            let reverse_path: Vec<&str> = reverse_path.iter().map(|s| s.as_str()).collect();
            let reverse_event = Event::build(&reverse_path, Some(payload.as_str()));

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("add", at_path).is_some() {
            todo!()
        } else if Self::starts_with("remove", at_path).is_some() {
            // get payload components
            let payload = Self::clone_payload_str(event, component_name)?;
            let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
            let regulator = Self::str_from_json(&payload_json, "regulator")?;
            let regulator_id = self.get_var_id(&regulator)?;
            let target = Self::str_from_json(&payload_json, "target")?;
            let target_id = self.get_var_id(&target)?;

            let original_regulation = self.get_regulation(&regulator_id, &target_id)?.clone();

            // perform the event, prepare the state-change variant
            self.remove_regulation(&regulator_id, &target_id)?;
            let state_change = event.clone();

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.path.remove(reverse_event.path.len() - 1);
            reverse_event.path.push("add".to_string());
            reverse_event.payload = Some(
                json!({
                    "regulator": regulator_id.as_str(),
                    "target": target_id.as_str(),
                    "sign": original_regulation.get_sign().to_string(),
                    "observable": original_regulation.is_observable(),
                })
                .to_string(),
            );

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("set_sign", at_path).is_some() {
            todo!()
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

        if Self::starts_with("update_position", at_path).is_some() {
            // get payload components
            let payload = Self::clone_payload_str(event, component_name)?;
            let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
            let layout_id_str = Self::str_from_json(&payload_json, "layout_id")?;
            let layout_id = self.get_layout_id(layout_id_str.as_str())?;
            let var_id_str = Self::str_from_json(&payload_json, "var_id")?;
            let var_id = self.get_var_id(var_id_str.as_str())?;
            let new_x = Self::float_from_json(&payload_json, "new_x")?;
            let new_y = Self::float_from_json(&payload_json, "new_y")?;
            let new_position = NodePosition(new_x, new_y);
            let original_position = self.get_node_position(&layout_id, &var_id)?.clone();

            if new_position == original_position {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant
            self.update_node_position(&layout_id, &var_id, new_x, new_y)?;
            let state_change = event.clone();

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(
                json!({
                    "layout_id": layout_id_str,
                    "var_id": var_id_str,
                    "new_x": original_position.0,
                    "new_y": original_position.1,
                })
                .to_string(),
            );

            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("add", at_path).is_some() {
            todo!()
        } else if Self::starts_with("remove", at_path).is_some() {
            todo!()
        } else {
            Self::throw_path_error(at_path, component_name)
        }
    }
}

impl SessionState for ModelState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        Self::assert_path_length(at_path, 2, "model")?;

        match at_path.first() {
            Some(&"variable") => self.perform_variable_event(event, &at_path[1..]),
            Some(&"regulation") => self.perform_regulation_event(event, &at_path[1..]),
            Some(&"layout") => self.perform_layout_event(event, &at_path[1..]),
            _ => Self::invalid_path_error(at_path),
        }
    }

    /// TODO: change this to make it a valid getter event API
    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        if !at_path.is_empty() {
            let msg = format!("`ModelState` cannot consume a path `{:?}`.", at_path);
            return AeonError::throw(msg);
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
        let payload = json!({
            "id": "b",
            "name": "b-name",
        })
        .to_string();
        let full_path = ["model", "variable", "add"];
        let event = Event::build(&full_path, Some(payload.as_str()));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        // check var was added
        assert_eq!(reg_state.num_vars(), 2);
        assert_eq!(reg_state.get_var_id("b").unwrap(), VarId::new("b").unwrap());
        check_reverse(reg_state, reg_state_orig, result, &["variable", "remove"]);
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
        let full_path = ["model", "variable", "remove"];
        let event = Event::build(&full_path, Some("a"));
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
        let new_name = "new_name";
        reg_state.add_var(var_id.clone(), original_name).unwrap();
        let reg_state_orig = reg_state.clone();
        assert_eq!(reg_state.get_var_name(&var_id).unwrap(), original_name);

        // test variable rename event
        let payload = json!({
            "id": var_id.as_str(),
            "new_name": new_name,
        })
        .to_string();
        let full_path = ["model", "variable", "set_name"];
        let event = Event::build(&full_path, Some(payload.as_str()));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        // check var was renamed
        assert_eq!(reg_state.get_var_name(&var_id).unwrap(), new_name);
        check_reverse(reg_state, reg_state_orig, result, &["variable", "set_name"]);
    }

    #[test]
    fn test_set_var_id_event() {
        let mut reg_state = ModelState::new();
        let var_id = reg_state.generate_var_id("a");
        reg_state.add_var(var_id.clone(), "a_name").unwrap();
        let reg_state_orig = reg_state.clone();

        // test id change event
        let new_id = reg_state.generate_var_id("b");
        let payload = json!({
            "original_id": var_id.as_str(),
            "new_id": new_id.as_str(),
        })
        .to_string();
        let full_path = ["model", "variable", "set_id"];
        let event = Event::build(&full_path, Some(payload.as_str()));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        // check id changed
        assert!(!reg_state.is_valid_var_id(&var_id));
        assert!(reg_state.is_valid_var_id(&new_id));
        check_reverse(reg_state, reg_state_orig, result, &["variable", "set_id"]);
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
        let full_path = ["model", "variable", "remove"];
        let event = Event::build(&full_path, Some("b"));
        assert!(reg_state.perform_event(&event, &full_path[1..]).is_err());
        assert_eq!(reg_state, reg_state_orig);

        // variable rename event with wrong id
        let payload = json!({"id": "b", "new_name": "x",}).to_string();
        let full_path = ["model", "variable", "set_name"];
        let event = Event::build(&full_path, Some(payload.as_str()));
        assert!(reg_state.perform_event(&event, &full_path[1..]).is_err());
        assert_eq!(reg_state, reg_state_orig);
    }

    #[test]
    fn test_add_reg_str_event() {
        let mut reg_state = ModelState::new();
        let var_id_a = reg_state.generate_var_id("a");
        reg_state.add_var(var_id_a, "a").unwrap();
        let var_id_a = reg_state.generate_var_id("b");
        reg_state.add_var(var_id_a, "b").unwrap();
        let reg_state_orig = reg_state.clone();

        // test regulation add event
        let full_path = ["model", "regulation", "add_by_str"];
        let event = Event::build(&full_path, Some("a -> b"));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        assert_eq!(reg_state.num_regulations(), 1);
        check_reverse(
            reg_state,
            reg_state_orig,
            result,
            &["regulation", "remove_by_str"],
        );
    }

    #[test]
    fn test_remove_reg_str_event() {
        let mut reg_state = ModelState::new();
        let var_id_a = reg_state.generate_var_id("a");
        reg_state.add_var(var_id_a, "a").unwrap();
        let var_id_a = reg_state.generate_var_id("b");
        reg_state.add_var(var_id_a, "b").unwrap();
        reg_state.add_regulation_by_str("a -> b").unwrap();
        let reg_state_orig = reg_state.clone();

        // test regulation add event
        let full_path = ["model", "regulation", "remove_by_str"];
        let event = Event::build(&full_path, Some("a -> b"));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        assert_eq!(reg_state.num_regulations(), 0);
        check_reverse(
            reg_state,
            reg_state_orig,
            result,
            &["regulation", "add_by_str"],
        );
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
            "layout_id": layout_id.as_str(),
            "var_id": var_id.as_str(),
            "new_x": 2.5,
            "new_y": 0.4,
        })
        .to_string();
        let full_path = ["model", "layout", "update_position"];
        let event = Event::build(&full_path, Some(payload.as_str()));
        let result = reg_state.perform_event(&event, &full_path[1..]).unwrap();

        // check position changed
        assert_eq!(
            reg_state.get_node_position(&layout_id, &var_id).unwrap(),
            &NodePosition(2.5, 0.4)
        );

        check_reverse(
            reg_state,
            reg_state_orig,
            result,
            &["layout", "update_position"],
        );
    }
}
