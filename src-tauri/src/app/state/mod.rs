use crate::app::event::UserAction;
use crate::app::DynError;

mod _consumed;
mod _state_app;
mod _state_atomic;
mod _state_map;

pub use _consumed::Consumed;
pub use _state_app::AppState;
pub use _state_atomic::AtomicState;
pub use _state_map::MapState;

pub type DynSessionState = Box<(dyn SessionState + Send + 'static)>;

pub trait SessionState {
    fn consume_event(&mut self, path: &[&str], action: &UserAction) -> Result<Consumed, DynError>;
}
