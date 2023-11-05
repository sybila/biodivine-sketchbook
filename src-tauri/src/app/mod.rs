use std::error::Error;

mod _aeon_app;
mod _aeon_error;
mod _undo_stack;
pub mod event;
pub mod state;

pub use _aeon_app::AeonApp;
pub use _aeon_error::AeonError;
pub use _undo_stack::UndoStack;

pub const EVENT_USER_ACTION: &str = "aeon-user-action";
pub const EVENT_STATE_CHANGE: &str = "aeon-state-change";

/// A [DynError] is a "generic" heap-allocated trait object which implements [std::error::Error].
///
/// You can convert most standard "typed" errors into [DynError]. If you want to
/// throw a general "runtime error" with no particular type, you can also use
/// [AeonError] (see [AeonError::throw] and [AeonError::throw_with_cause]).
pub type DynError = Box<(dyn Error + 'static)>;
