use crate::analysis::analysis_state::AnalysisState;
use crate::app::event::{Event, SessionMessage, StateChange};
use crate::app::state::_undo_stack::UndoStack;
use crate::app::state::{Consumed, SessionHelper, SessionState, StackSession};
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
}

impl StackSession for AnalysisSession {
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

            // no backend response is expected, but we must send refresh event to inform frontend
            // about the state change
            let sketch_data = SketchData::new_from_sketch(self.analysis_state.get_sketch());
            let payload = sketch_data.to_json_str();
            let state_change = StateChange {
                events: vec![Event::build(&["analysis", "get_sketch"], Some(&payload))],
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

    fn undo_stack_mut(&mut self) -> &mut UndoStack {
        &mut self.undo_stack
    }

    fn undo_stack(&self) -> &UndoStack {
        &self.undo_stack
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
