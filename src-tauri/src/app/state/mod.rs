use crate::app::event::{Event, StateChange, UserAction};
use crate::app::DynError;

mod _consumed;
mod _state_app;
mod _state_atomic;
mod _state_map;
pub mod _undo_stack;

/// Declares state objects that are unique to the sketchbook editor window.
pub mod editor;

pub use _consumed::Consumed;
pub use _state_app::AppState;
pub use _state_atomic::AtomicState;
//pub use _state_map::MapState;

pub type DynSessionState = Box<(dyn SessionState + Send + 'static)>;
pub type DynSession = Box<(dyn Session + Send + 'static)>;

pub trait SessionState {
    /// Modify the session state using the provided `event`. The possible outcomes are
    /// described by [Consumed].
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError>;

    /// "Read" session state into an event without modifying it.
    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError>;
}

pub trait Session: SessionState {
    /// Perform a user action on this session state object. This usually involves propagating
    /// the events to the internal [SessionState] objects and collecting the results into a
    /// single [StateChange] entry.
    fn perform_action(&mut self, action: &UserAction) -> Result<StateChange, DynError>;

    /// Returns the string identifier of this particular session. Each session identifier must
    /// be unique within the application.
    fn id(&self) -> &str;
}
