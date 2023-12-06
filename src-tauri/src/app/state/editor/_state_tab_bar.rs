use crate::app::event::Event;
use crate::app::state::{AtomicState, Consumed, SessionState};
use crate::app::{AeonError, DynError};

#[derive(Default)]
pub struct TabBarState {
    active: AtomicState<u32>,
    pinned: AtomicState<Vec<u32>>,
}

impl SessionState for TabBarState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        if at_path.is_empty() {
            return AeonError::throw("`TabBar` cannot process an empty path.");
        }

        match at_path[0] {
            "active" => self.active.perform_event(event, &at_path[1..]),
            "pinned" => self.pinned.perform_event(event, &at_path[1..]),
            it => AeonError::throw(format!("Unknown path in `TabBar`: `{}`", it)),
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        if at_path.is_empty() {
            return AeonError::throw("`TabBar` cannot process an empty path.");
        }

        match at_path[0] {
            "active" => self.active.refresh(full_path, &at_path[1..]),
            "pinned" => self.pinned.refresh(full_path, &at_path[1..]),
            it => AeonError::throw(format!("Unknown path in `TabBar`: `{}`", it)),
        }
    }
}
