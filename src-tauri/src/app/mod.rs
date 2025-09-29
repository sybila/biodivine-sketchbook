use std::error::Error;

mod _aeon_app;
mod _aeon_error;
pub mod event;
pub mod event_wrappers;
pub mod state;

pub use _aeon_app::AeonApp;
pub use _aeon_error::AeonError;

/// Label for frontend events that are changing the app state.
pub const AEON_ACTION: &str = "aeon-action";

/// Label for backend events that are notifying about a state change.
pub const AEON_VALUE: &str = "aeon-value";

/// Label for frontend events that are requesting a value retransmission.
pub const AEON_REFRESH: &str = "aeon-refresh";

/// Label for backend-only events for simple "request-answer" messaging between different sessions
/// to exchange predefined values.
pub const AEON_MESSAGE: &str = "aeon-message";

/// ID of the default initial editor session.
pub const DEFAULT_SESSION_ID: &str = "editor-1";

/// ID of the initial window of the default editor session.
pub const DEFAULT_WINDOW_ID: &str = "editor";

/// A [DynError] is a "generic" heap-allocated trait object which implements [std::error::Error].
///
/// You can convert most standard "typed" errors into [DynError]. If you want to
/// throw a general "runtime error" with no particular type, you can also use
/// [AeonError] (see [AeonError::throw] and [AeonError::throw_with_source]).
pub type DynError = Box<dyn Error + 'static>;
