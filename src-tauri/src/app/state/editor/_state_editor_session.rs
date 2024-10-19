use crate::app::event::{Event, SessionMessage, StateChange};
use crate::app::state::_undo_stack::UndoStack;
use crate::app::state::editor::TabBarState;
use crate::app::state::{Consumed, SessionHelper, SessionState, StackSession};
use crate::app::{AeonError, DynError};
use crate::debug;
use crate::sketchbook::Sketch;

/// The state of one editor session.
///
/// An editor session is the "main" app session where a model is created/edited and from which
/// different analysis sessions can be started.
pub struct EditorSession {
    id: String,
    undo_stack: UndoStack,
    tab_bar: TabBarState,
    sketch: Sketch,
}

impl EditorSession {
    pub fn new(id: &str) -> EditorSession {
        EditorSession {
            id: id.to_string(),
            undo_stack: UndoStack::default(),
            tab_bar: TabBarState::default(),
            sketch: Sketch::default(),
        }
    }
}

impl StackSession for EditorSession {
    fn process_message(
        &mut self,
        message: &SessionMessage,
    ) -> Result<(Option<SessionMessage>, Option<StateChange>), DynError> {
        let path = message.message.path.clone();

        // If the state changed due to message processing, we'll have to reset the undo-redo stack
        // (but we do not use messages that make these changes often)

        // todo: make this `mut` when we have some cases here that could mutate state
        let reset_stack = false;

        // request from new Analysis session for sending a sketch
        let result = if path == vec!["send_sketch".to_string()] {
            let sketch_string = self.sketch.to_custom_json();
            let response_msg = SessionMessage {
                message: Event::build(&["sketch_sent"], Some(&sketch_string)),
            };
            // response message; but no change in state for frontend
            Ok((Some(response_msg), None))
        } else {
            let error_msg = format!("`EditorSession` cannot process path {:?}.", path);
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

    fn undo_stack_mut(&mut self) -> &mut UndoStack {
        &mut self.undo_stack
    }

    fn undo_stack(&self) -> &UndoStack {
        &self.undo_stack
    }
}

impl SessionHelper for EditorSession {}

impl SessionState for EditorSession {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        if let Some(at_path) = Self::starts_with("undo_stack", at_path) {
            self.undo_stack.perform_event(event, at_path)
        } else if let Some(at_path) = Self::starts_with("tab_bar", at_path) {
            self.tab_bar.perform_event(event, at_path)
        } else if let Some(at_path) = Self::starts_with("sketch", at_path) {
            self.sketch.perform_event(event, at_path)
        } else {
            Self::invalid_path_error_generic(at_path)
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        if let Some(at_path) = Self::starts_with("undo_stack", at_path) {
            self.undo_stack.refresh(full_path, at_path)
        } else if let Some(at_path) = Self::starts_with("tab_bar", at_path) {
            self.tab_bar.refresh(full_path, at_path)
        } else if let Some(at_path) = Self::starts_with("sketch", at_path) {
            self.sketch.refresh(full_path, at_path)
        } else {
            Self::invalid_path_error_generic(at_path)
        }
    }
}
