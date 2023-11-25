use crate::app::event::UserAction;
use crate::app::state::{Consumed, SessionState};
use crate::app::DynError;
use crate::sketchbook::RegulationsState;

impl SessionState for RegulationsState {
    /// TODO
    fn consume_event(&mut self, path: &[&str], action: &UserAction) -> Result<Consumed, DynError> {
        todo!()
    }
}
