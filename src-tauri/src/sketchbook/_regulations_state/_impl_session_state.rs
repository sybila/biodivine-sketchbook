use crate::app::event::Event;
use crate::app::state::{Consumed, SessionState};
use crate::app::{AeonError, DynError};
use crate::sketchbook::RegulationsState;
use serde_json::json;

/// TODO: expand this.
/// List of events we will need, with their path, payload format, and path to the corresponding
/// event for front-end. Payloads are single string values, or a json object with multiple string
/// fields.
/// ["variable", "add"]; payload = {"id": var_id, "name": name}; `add-variable`
/// ["variable", "remove"]; payload = var_id, `remove-variable`
/// ["variable", "set_name"]; payload = {"id": var_id, "new_name": new_name}; `set-name`
/// ["variable", "set_id"],; payload = {"original_id": original_id, "new_id": new_id}; `set-id`
/// ["regulation", "add-by-str"]; payload = regulation_str; `add-regulation-by-str`
/// ["regulation", "add"]; payload = {"regulator": reg_id, "target": target_id, "sign": sign, "observable": observ}; `add-regulation`
/// ["regulation", "remove"]; payload = {"regulator": reg_id, "target": target_id}, `remove-regulation`
/// ["layout", "add"]; payload = {"id": layout_id, "name": name}; `add-layout`
/// ["layout", "set_name"]; payload = {"id": layout_id, "new_name": new_name}; `set-layout-name`
/// ["layout", "update_position"]; payload = {"layout_id": layout_id, "var_id": var_id, "new_x": x_val, "new_y": y_val}; `update-position`

/// Functionality and shorthands related for the `SessionState` trait.
impl RegulationsState {
    /// Shorthand to get and clone a payload of an event. Errors if payload is empty.
    /// The `component` specifies which part of the state should be mentioned in the error.
    /// In future we may consider moving this elsewhere.
    fn clone_payload_str(event: &Event, component: &str) -> Result<String, DynError> {
        let payload = event.payload.clone().ok_or(format!(
            "Event to `{component}` cannot carry empty payload."
        ))?;
        Ok(payload)
    }

    /// Shorthand to get a given field from json object. Errors if no such field or bad format.
    fn get_from_json(json_payload: &serde_json::Value, field_name: &str) -> Result<String, String> {
        let value = json_payload[field_name]
            .as_str()
            .ok_or(format!("Payload {json_payload} is invalid"))?
            .trim_matches('\"')
            .to_string();
        Ok(value)
    }

    /// Shorthand to get and clone a full path of an event.
    /// In future we may consider moving this elsewhere.
    fn clone_path(event: &Event) -> Vec<String> {
        event.path.clone()
    }

    /// Shorthand to assert that path has given length and return typesafe `DynError` otherwise.
    /// The `component` specifies which component of the state should be mentioned in the error.
    fn assert_path_length(path: &[&str], length: usize, component: &str) -> Result<(), DynError> {
        if path.len() != length {
            return AeonError::throw(format!("`{component}` cannot consume a path `{:?}`.", path));
        }
        Ok(())
    }

    /// Consume events related to `variables` component of this `RegulationsState`.
    fn consume_variable_event(
        &mut self,
        path: &[&str],
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "RegulationState/variable";
        println!("Consuming event {:?} at {component_name}", event);
        Self::assert_path_length(path, 1, component_name)?;

        match path.first() {
            Some(&"add") => {
                // get payload components
                let payload = Self::clone_payload_str(event, component_name)?;
                let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
                let var_id_str = Self::get_from_json(&payload_json, "id")?;
                let name = Self::get_from_json(&payload_json, "name")?;

                // perform the event, prepare the state-change variant
                self.add_var_by_str(var_id_str.as_str(), name.as_str())?;
                let state_change = Event::build(&["add-variable"], Some(&payload));

                // prepare the reverse event
                let mut reverse_path = Self::clone_path(event);
                reverse_path.remove(reverse_path.len() - 1);
                reverse_path.push("remove".to_string());
                let reverse_path: Vec<&str> = reverse_path.iter().map(|s| s.as_str()).collect();
                let reverse_event = Event::build(&reverse_path, Some(var_id_str.as_str()));

                Ok(Consumed::Reversible {
                    state_change,
                    perform_reverse: (event.clone(), reverse_event),
                })
            }
            Some(&"remove") => {
                let var_id_str = Self::clone_payload_str(event, component_name)?;
                self.remove_var_by_str(var_id_str.as_str())?;
                let state_change = Event::build(&["remove-variable"], Some(var_id_str.as_str()));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"set_name") => {
                // get payload components
                let payload = Self::clone_payload_str(event, component_name)?;
                let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
                let var_id_str = Self::get_from_json(&payload_json, "id")?;
                let var_id = self.get_var_id(var_id_str.as_str())?;
                let new_name = Self::get_from_json(&payload_json, "new_name")?;
                let original_name = self.get_var_name(&var_id)?.to_string();

                if new_name == original_name {
                    return Ok(Consumed::NoChange);
                }

                // perform the event, prepare the state-change variant
                self.set_var_name(&var_id, new_name.as_str())?;
                let state_change = Event::build(&["set-name"], Some(&payload));

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
            }
            Some(&"set_id") => {
                // get payload components
                let payload = Self::clone_payload_str(event, component_name)?;
                let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
                let original_id = Self::get_from_json(&payload_json, "original_id")?;
                let new_id = Self::get_from_json(&payload_json, "new_id")?;

                if original_id == new_id {
                    return Ok(Consumed::NoChange);
                }

                // perform the event, prepare the state-change variant
                self.set_var_id_by_str(&original_id, &new_id)?;
                let state_change = Event::build(&["set-id"], Some(&payload));

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
            }
            _ => AeonError::throw(format!(
                "`RegulationState/variable` cannot consume a path `{:?}`.",
                path
            )),
        }
    }

    /// Consume events related to `regulations` component of this `RegulationsState`.
    fn consume_regulation_event(
        &mut self,
        path: &[&str],
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "RegulationState/regulation";
        println!("Consuming event {:?} at {component_name}", event);
        Self::assert_path_length(path, 1, component_name)?;

        match path.first() {
            Some(&"add") => {
                todo!()
            }
            Some(&"remove") => {
                todo!()
            }
            _ => AeonError::throw(format!(
                "`RegulationState/regulation` cannot consume a path `{:?}`.",
                path
            )),
        }
    }

    /// Consume events related to `layouts` component of this `RegulationsState`.
    fn consume_layout_event(&mut self, path: &[&str], event: &Event) -> Result<Consumed, DynError> {
        let component_name = "RegulationState/layout";
        println!("Consuming event {:?} at {component_name}", event);
        Self::assert_path_length(path, 1, component_name)?;

        match path.first() {
            Some(&"add") => {
                todo!()
            }
            Some(&"remove") => {
                todo!()
            }
            _ => AeonError::throw(format!(
                "`RegulationState/layout` cannot consume a path `{:?}`.",
                path
            )),
        }
    }
}

impl SessionState for RegulationsState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        Self::assert_path_length(at_path, 2, "RegulationState")?;

        match at_path.first() {
            Some(&"variable") => self.consume_variable_event(&at_path[1..], event),
            Some(&"regulation") => self.consume_regulation_event(&at_path[1..], event),
            Some(&"layout") => self.consume_layout_event(&at_path[1..], event),
            _ => AeonError::throw(format!(
                "`RegulationState` cannot consume a path `{:?}`.",
                at_path
            )),
        }
    }

    /// TODO: how to do this - return whole `RegulationsState` or individual components?
    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        if !at_path.is_empty() {
            let msg = format!("Atomic state cannot consume a path `{:?}`.", at_path);
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
    use crate::sketchbook::{RegulationsState, VarId};
    use serde_json::json;

    #[test]
    fn test_add_var_event() {
        let mut reg_state = RegulationsState::new();
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
        let event = Event::build(
            &["regulations_state", "variable", "add"],
            Some(payload.as_str()),
        );
        let result = reg_state
            .perform_event(&event, &["variable", "add"])
            .unwrap();

        // check var was added
        assert_eq!(reg_state.num_vars(), 2);
        assert_eq!(reg_state.get_var_id("b").unwrap(), VarId::new("b").unwrap());

        // assert that the reverse event is correct
        match result {
            Consumed::Reversible {
                perform_reverse: (_, reverse),
                ..
            } => {
                reg_state
                    .perform_event(&reverse, &["variable", "remove"])
                    .unwrap();
                assert_eq!(reg_state, reg_state_orig);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_remove_var_event() {
        let mut reg_state = RegulationsState::new();
        let var_id_a = reg_state.generate_var_id("a");
        reg_state.add_var(var_id_a, "a").unwrap();
        reg_state.add_regulation_by_str("a -> a").unwrap();
        assert_eq!(reg_state.num_vars(), 1);
        assert_eq!(reg_state.num_regulations(), 1);

        // test variable remove event
        let event = Event::build(&["regulations_state", "variable", "remove"], Some("a"));
        let result = reg_state
            .perform_event(&event, &["variable", "remove"])
            .unwrap();

        // check var was removed
        assert_eq!(reg_state.num_vars(), 0);
        assert_eq!(reg_state.num_regulations(), 0);

        // assert that it is irreversible (at the moment)
        assert!(matches!(result, Consumed::Irreversible { .. }));
    }

    #[test]
    fn test_set_var_name_event() {
        let mut reg_state = RegulationsState::new();
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
        let event = Event::build(
            &["regulations_state", "variable", "set_name"],
            Some(payload.as_str()),
        );
        let result = reg_state
            .perform_event(&event, &["variable", "set_name"])
            .unwrap();

        // check var was renamed
        assert_eq!(reg_state.get_var_name(&var_id).unwrap(), new_name);

        // assert that the reverse event is correct
        match result {
            Consumed::Reversible {
                perform_reverse: (_, reverse),
                ..
            } => {
                reg_state
                    .perform_event(&reverse, &["variable", "set_name"])
                    .unwrap();
                assert_eq!(reg_state.get_var_name(&var_id).unwrap(), original_name);
                assert_eq!(reg_state, reg_state_orig);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_set_var_id_event() {
        let mut reg_state = RegulationsState::new();
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
        let event = Event::build(
            &["regulations_state", "variable", "set_id"],
            Some(payload.as_str()),
        );
        let result = reg_state
            .perform_event(&event, &["variable", "set_id"])
            .unwrap();

        // check id changed
        assert!(!reg_state.is_valid_var_id(&var_id));
        assert!(reg_state.is_valid_var_id(&new_id));

        // assert that the reverse event is correct
        match result {
            Consumed::Reversible {
                perform_reverse: (_, reverse),
                ..
            } => {
                reg_state
                    .perform_event(&reverse, &["variable", "set_id"])
                    .unwrap();
                assert_eq!(reg_state, reg_state_orig);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_invalid_var_events() {
        let mut reg_state = RegulationsState::new();
        let var_id = reg_state.generate_var_id("a");
        reg_state.add_var(var_id.clone(), "a-name").unwrap();
        let reg_state_orig = reg_state.clone();

        // adding variable `a` again
        let event = Event::build(&["regulations_state", "variable", "add"], Some("a"));
        assert!(reg_state
            .perform_event(&event, &["variable", "add"])
            .is_err());
        assert_eq!(reg_state, reg_state_orig);

        // removing variable with wrong id
        let event = Event::build(&["regulations_state", "variable", "remove"], Some("b"));
        assert!(reg_state
            .perform_event(&event, &["variable", "remove"])
            .is_err());
        assert_eq!(reg_state, reg_state_orig);

        // variable rename event with wrong id
        let payload = json!({"id": "b","new_name": "x",}).to_string();
        let event = Event::build(
            &["regulations_state", "variable", "set_name"],
            Some(payload.as_str()),
        );
        assert!(reg_state
            .perform_event(&event, &["variable", "set_name"])
            .is_err());
        assert_eq!(reg_state, reg_state_orig);
    }
}
