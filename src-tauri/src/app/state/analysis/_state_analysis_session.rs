use crate::analysis::AnalysisState;
use crate::app::event::{Event, SessionMessage, StateChange, UserAction};
use crate::app::state::_undo_stack::UndoStack;
use crate::app::state::{Consumed, Session, SessionHelper, SessionState};
use crate::app::{AeonError, DynError};
use crate::debug;
use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::{JsonSerde, Sketch};

/// The state of one editor session.
///
/// An analysis session is the session where the process of the inference is run on a given model.
pub struct AnalysisSession {
    id: String,
    undo_stack: UndoStack,
    analysis_state: AnalysisState,
}

impl AnalysisSession {
    pub fn new(id: &str) -> AnalysisSession {
        AnalysisSession {
            id: id.to_string(),
            undo_stack: UndoStack::default(),
            analysis_state: AnalysisState::new_empty(),
        }
    }

    fn perform_action(
        &mut self,
        action: &UserAction,
        ignore_stack: bool,
    ) -> Result<StateChange, DynError> {
        // Events that need to be consume (last to first) in order to complete this action.
        let mut to_perform = action.events.clone();
        to_perform.reverse();

        // The events representing successful state changes.
        let mut state_changes: Vec<Event> = Vec::new();
        // The events that can be used to create a redo stack entry if the action is reversible.
        let mut reverse: Option<Vec<(Event, Event)>> =
            if ignore_stack { None } else { Some(Vec::new()) };
        let mut reset_stack = false;

        while let Some(event) = to_perform.pop() {
            let event_path = event.path.iter().map(|it| it.as_str()).collect::<Vec<_>>();
            debug!(
                "Executing event to session {}: `{:?}`.",
                self.id, event_path
            );
            let result = match self.perform_event(&event, &event_path) {
                Ok(result) => result,
                Err(error) => {
                    // TODO:
                    //  We should probably first emit the state change and then the
                    //  error, because now we are losing state of compound actions that fail.
                    return Err(error);
                }
            };
            match result {
                Consumed::Reversible {
                    state_change,
                    perform_reverse,
                } => {
                    state_changes.push(state_change);
                    if let Some(reverse) = reverse.as_mut() {
                        // If we can reverse this action, save the events.
                        reverse.push(perform_reverse);
                    }
                }
                Consumed::Irreversible {
                    state_change,
                    reset,
                } => {
                    state_changes.push(state_change);
                    if reset {
                        // We cannot reverse this event, but the rest can be reversed.
                        reverse = None;
                        reset_stack = true;
                    }
                }
                Consumed::Restart(mut events) => {
                    // Just push the new events to the execution stack and continue
                    // to the next event.
                    events.reverse();
                    while let Some(e) = events.pop() {
                        to_perform.push(e);
                    }
                }
                Consumed::InputError(error) => {
                    // TODO:
                    //  The same as above. We should report this as a separate event from the
                    //  state change that was performed.
                    return Err(error);
                }
                Consumed::NoChange => {
                    // Do nothing.
                }
            }
        }

        // If the action is not irreversible, we should add an entry to the undo stack.
        if let Some(events) = reverse {
            if !events.is_empty() {
                // Only add undo action if the stack is not empty.
                let mut perform = Vec::new();
                let mut reverse = Vec::new();
                for (p, r) in events {
                    perform.push(p);
                    reverse.push(r);
                }
                // Obviously, the "reverse" events need to be execute in the opposite order
                // compared to the "perform" events.
                reverse.reverse();
                let perform = UserAction { events: perform };
                let reverse = UserAction { events: reverse };
                if !self.undo_stack.do_action(perform, reverse) {
                    // TODO: Not match we can do here, maybe except issuing a warning.
                    self.undo_stack.clear();
                }

                // Notify about the changes in the stack state.
                // TODO: Maybe we don't need to emit this always.
                self.append_stack_updates(&mut state_changes);
            }
        } else if !ignore_stack && reset_stack {
            debug!(
                "Back stack (of session {}) cleared due to irreversible action.",
                self.id
            );
            self.undo_stack.clear();
        }

        Ok(StateChange {
            events: state_changes,
        })
    }

    fn append_stack_updates(&self, state_changes: &mut Vec<Event>) {
        let can_undo = serde_json::to_string(&self.undo_stack.can_undo());
        let can_redo = serde_json::to_string(&self.undo_stack.can_redo());
        state_changes.push(Event::build(
            &["undo_stack", "can_undo"],
            Some(can_undo.unwrap().as_str()),
        ));
        state_changes.push(Event::build(
            &["undo_stack", "can_redo"],
            Some(can_redo.unwrap().as_str()),
        ));
    }
}

impl Session for AnalysisSession {
    fn perform_action(&mut self, action: &UserAction) -> Result<StateChange, DynError> {
        // Explicit test for undo-stack actions.
        // TODO:
        //  Figure out a nicer way to do this. Probably modify the `Consumed` enum?
        //  We basically need a way to say "restart with these events, but as an
        //  Irreversible action that won't reset the stack."
        'undo: {
            if action.events.len() == 1 {
                let event = &action.events[0];
                if event.path.len() == 2 && event.path[0] == "undo_stack" {
                    let action = match event.path[1].as_str() {
                        "undo" => {
                            let Some(undo) = self.undo_stack.undo_action() else {
                                return AeonError::throw("Nothing to undo.");
                            };
                            undo
                        }
                        "redo" => {
                            let Some(redo) = self.undo_stack.redo_action() else {
                                return AeonError::throw("Nothing to redo.");
                            };
                            redo
                        }
                        _ => break 'undo,
                    };
                    let mut state_change = self.perform_action(&action, true)?;
                    self.append_stack_updates(&mut state_change.events);
                    return Ok(state_change);
                }
            }
        }
        self.perform_action(action, false)
    }

    fn process_message(
        &mut self,
        message: &SessionMessage,
    ) -> Result<(Option<SessionMessage>, Option<StateChange>), DynError> {
        let path = message.message.path.clone();

        // if the state changed due to message processing, we'll have to reset the undo-redo stack
        // do not use messages that make these changes often
        let mut reset_stack = false;

        // message with sketch data sent from Editor session
        let result = if path == vec!["sketch_sent".to_string()] {
            if let Some(sketch_payload) = message.message.payload.clone() {
                let sketch = Sketch::from_json_str(&sketch_payload)?;
                reset_stack = true;
                self.analysis_state.set_sketch(sketch);
            } else {
                panic!("Message `sketch_sent` must always carry a payload.")
            }
            // no response is expected, but we must inform frontend about state change
            let sketch_data = SketchData::new_from_sketch(self.analysis_state.get_sketch());
            let payload = sketch_data.to_json_str();
            let state_change = StateChange {
                events: vec![Event::build(
                    &["analysis", "sketch_changed"],
                    Some(&payload),
                )],
            };
            Ok((None, Some(state_change)))
        } else {
            let error_msg = format!("`AnalysisSession` cannot process path {:?}.", path);
            AeonError::throw(error_msg)
        };

        if reset_stack {
            debug!(
                "Back stack (of session {}) cleared due to backend change.",
                self.id
            );
            self.undo_stack.clear();
        }
        result
    }

    fn id(&self) -> &str {
        self.id.as_str()
    }
}

impl SessionHelper for AnalysisSession {}

impl SessionState for AnalysisSession {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        if let Some(at_path) = Self::starts_with("undo_stack", at_path) {
            self.undo_stack.perform_event(event, at_path)
        } else if let Some(at_path) = Self::starts_with("analysis", at_path) {
            self.analysis_state.perform_event(event, at_path)
        } else {
            Self::invalid_path_error_generic(at_path)
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        if let Some(at_path) = Self::starts_with("undo_stack", at_path) {
            self.undo_stack.refresh(full_path, at_path)
        } else if let Some(at_path) = Self::starts_with("analysis", at_path) {
            self.analysis_state.refresh(full_path, at_path)
        } else {
            Self::invalid_path_error_generic(at_path)
        }
    }
}
