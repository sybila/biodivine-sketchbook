use crate::app::event::{Event, SessionMessage, StateChange, UserAction};
use crate::app::{AeonError, DynError};

mod _consumed;
mod _state_app;
mod _state_atomic;
mod _state_map;
pub mod _undo_stack;

/// Declares top-level state objects that are unique to the sketchbook editor session.
pub mod editor;
/// Declares top-level state objects that are unique to the sketchbook inference session.
pub mod inference;

use crate::app::state::_undo_stack::UndoStack;
use crate::debug;
pub use _consumed::Consumed;
pub use _state_app::AppState;
pub use _state_atomic::AtomicState;
//pub use _state_map::MapState;

pub type DynSessionState = Box<dyn SessionState + Send + 'static>;
pub type DynSession = Box<dyn StackSession + Send + 'static>;

/// Wrapper to escape a string message and wrap it in quotes (in a crude way).
fn escape_string_json(message: &str) -> String {
    serde_json::Value::String(message.to_string()).to_string()
}

pub trait SessionState {
    /// Modify the session state using the provided `event`. The possible outcomes are
    /// described by [Consumed].
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError>;

    /// "Read" session state into an event without modifying it.
    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError>;
}

pub trait SessionHelper {
    /// A utility function which checks if `at_path` starts with a specific first segment.
    /// If yes, returns the remaining part of the path.
    fn starts_with<'a, 'b>(prefix: &str, at_path: &'a [&'b str]) -> Option<&'a [&'b str]> {
        if let Some(x) = at_path.first() {
            if x == &prefix {
                Some(&at_path[1..])
            } else {
                None
            }
        } else {
            None
        }
    }

    /// A utility function which checks if `at_path` is exactly
    fn matches(expected: &[&str], at_path: &[&str]) -> bool {
        if expected.len() != at_path.len() {
            return false;
        }

        for (a, b) in expected.iter().zip(at_path) {
            if a != b {
                return false;
            }
        }

        true
    }

    /// A utility function which emits a generic "invalid path" error.
    fn invalid_path_error_generic<T>(at_path: &[&str]) -> Result<T, DynError> {
        AeonError::throw(format!(
            "`{}` cannot process path `{:?}`.",
            std::any::type_name::<Self>(),
            at_path
        ))
    }

    /// A utility function which emits a "invalid path" error mentioning specific state's `component`.
    fn invalid_path_error_specific<T>(path: &[&str], component: &str) -> Result<T, DynError> {
        AeonError::throw(format!("`{component}` cannot process path `{path:?}`."))
    }

    /// A utility function to get and clone a payload of an event. Errors if payload is empty.
    ///
    /// The `component` specifies which component of the state should be mentioned in the error.
    /// This might be useful to directly mention relevant fields of more complex types.
    fn clone_payload_str(event: &Event, component: &str) -> Result<String, DynError> {
        let payload = event.payload.clone().ok_or(format!(
            "This event to `{component}` cannot carry empty payload."
        ))?;
        Ok(payload)
    }

    /// A utility function to assert that path has a given length, or emit a `DynError` otherwise.
    ///
    /// The `component` specifies the state's component that should be mentioned in the error.
    /// This might be useful to directly mention relevant fields of more complex types.
    fn assert_path_length(path: &[&str], length: usize, component: &str) -> Result<(), DynError> {
        if path.len() != length {
            return AeonError::throw(format!("`{component}` cannot process path `{path:?}`."));
        }
        Ok(())
    }

    /// A utility function to assert that payload is empty - otherwise, `DynError` is emitted.
    ///
    /// The `component` specifies which component of the state should be mentioned in the error.
    /// This might be useful to directly mention relevant fields of more complex types.
    fn assert_payload_empty(event: &Event, component: &str) -> Result<(), DynError> {
        if event.payload.is_some() {
            let message = format!("This event to `{component}` must carry empty payload.");
            return AeonError::throw(message);
        }
        Ok(())
    }
}

/// A Session with a [UndoStack] with events.
///
/// Sessions perform user actions, or communicate with different sessions via messages.
pub trait StackSession: SessionState {
    /// Perform a user action on this session state object. This usually involves propagating
    /// the events to the internal [SessionState] objects and collecting the results into a
    /// single [StateChange] entry.
    ///
    /// In this top-level method, we explicitly test for undo-stack actions. Once that is done,
    /// the processing continues via [StackSession::perform_categorized_action].
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
                            let Some(undo) = self.undo_stack_mut().undo_action() else {
                                return AeonError::throw("Nothing to undo.");
                            };
                            undo
                        }
                        "redo" => {
                            let Some(redo) = self.undo_stack_mut().redo_action() else {
                                return AeonError::throw("Nothing to redo.");
                            };
                            redo
                        }
                        _ => break 'undo,
                    };
                    let mut state_change = self.perform_categorized_action(&action, true)?;
                    self.append_stack_updates(&mut state_change.events);
                    return Ok(state_change);
                }
            }
        }
        self.perform_categorized_action(action, false)
    }

    /// Perform a user action on this session state object, with additional information whether
    /// the action should bypass the undo-redo stack.
    ///
    /// This method assumes the action was already categorized into one of `undo` (stack should be
    /// bypassed) or `regular` (goes to the undo stack).
    /// If you want to run the full process including categorizing the action, use [StackSession::perform_action].
    fn perform_categorized_action(
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
                self.id(),
                event_path
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
                Consumed::IrreversibleWithWarning {
                    state_change,
                    reset,
                    warning,
                } => {
                    // First add a new state change event with the warning
                    let json_message = escape_string_json(&warning); // Excape the message in quotes.
                    let warning_state_change = Event::build(&["warning"], Some(&json_message));
                    state_changes.push(warning_state_change);

                    // And also add the original state change event
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
                if !self.undo_stack_mut().do_action(perform, reverse) {
                    // TODO: Not much we can do here, maybe except issuing a warning.
                    self.undo_stack_mut().clear();
                }

                // Notify about the changes in the stack state.
                // TODO: Maybe we don't need to emit this always.
                self.append_stack_updates(&mut state_changes);
            }
        } else if !ignore_stack && reset_stack {
            debug!(
                "Back stack (of session {}) cleared due to irreversible action.",
                self.id()
            );
            self.undo_stack_mut().clear();
        }

        Ok(StateChange {
            events: state_changes,
        })
    }

    fn append_stack_updates(&self, state_changes: &mut Vec<Event>) {
        let can_undo = serde_json::to_string(&self.undo_stack().can_undo());
        let can_redo = serde_json::to_string(&self.undo_stack().can_redo());
        state_changes.push(Event::build(
            &["undo_stack", "can_undo"],
            Some(can_undo.unwrap().as_str()),
        ));
        state_changes.push(Event::build(
            &["undo_stack", "can_redo"],
            Some(can_redo.unwrap().as_str()),
        ));
    }

    /// Process a message sent to this session state object.
    ///
    /// Depending on the message, an optional "response" [SessionMessage] might be returned.
    /// This will be sent to the sender of the original message.
    /// Similarly, if the processing of the message caused some changes to the state, an optional
    /// "refresh" [SessionMessage] should be returned to then update the frontend.
    fn process_message(
        &mut self,
        message: &SessionMessage,
    ) -> Result<(Option<SessionMessage>, Option<StateChange>), DynError>;

    /// Returns the string identifier of this particular session. Each session identifier must
    /// be unique within the application.
    fn id(&self) -> &str;

    /// Returns an immutable reference to session's undo stack.
    fn undo_stack(&self) -> &UndoStack;

    /// Returns a mutable reference to session's undo stack.
    fn undo_stack_mut(&mut self) -> &mut UndoStack;
}
