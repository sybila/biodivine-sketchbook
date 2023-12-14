use crate::app::event::Event;
use crate::app::state::{AtomicState, Consumed, SessionHelper, SessionState};
use crate::app::DynError;

#[derive(Default)]
pub struct TabBarState {
    active: AtomicState<u32>,
    pinned: AtomicState<Vec<u32>>,
}

impl SessionHelper for TabBarState {}

impl SessionState for TabBarState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        if let Some(at_path) = Self::starts_with("active", at_path) {
            self.active.perform_event(event, at_path)
        } else if let Some(at_path) = Self::starts_with("pinned", at_path) {
            self.pinned.perform_event(event, at_path)
        } else {
            Self::invalid_path_error(at_path)
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        if let Some(at_path) = Self::starts_with("active", at_path) {
            self.active.refresh(full_path, at_path)
        } else if let Some(at_path) = Self::starts_with("pinned", at_path) {
            self.pinned.refresh(full_path, at_path)
        } else {
            Self::invalid_path_error(at_path)
        }
    }
}
