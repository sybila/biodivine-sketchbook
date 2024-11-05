use crate::app::event::{SessionMessage, StateChange, UserAction};
use crate::app::event_wrappers::AeonMessage;
use crate::app::state::DynSession;
use crate::app::{AeonApp, AeonError, DynError, AEON_MESSAGE, AEON_VALUE};
use crate::warning;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use tauri::{Manager, Window};

/// [AppState] implements mapping between session IDs and session objects.
///
/// Specifically, it ensures:
///  - Synchronization by locking the app state.
///  - Ability to add/remove windows as they are opened/closed.
///  - Error handling and retransmission of events.
///
/// As such, [AppState] does not actually implement the [super::SessionState] trait.
pub struct AppState {
    // Assigns a state object to every session.
    session_state: Mutex<HashMap<String, DynSession>>,
    // Assigns a session ID to every window.
    window_to_session: Mutex<HashMap<String, String>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            session_state: Mutex::new(HashMap::new()),
            window_to_session: Mutex::new(HashMap::new()),
        }
    }
}

impl AppState {
    /// Register the newly created session to the AppState.
    pub fn session_created(&self, id: &str, session: impl Into<DynSession>) {
        let mut guard = self.session_state.lock().unwrap_or_else(|_e| {
            panic!("Main application state is poisoned. Cannot recover.");
        });
        let session_map = guard.deref_mut();
        if session_map.contains_key(id) {
            // TODO: This should be a normal error.
            panic!("Session already exists.");
        }
        session_map.insert(id.to_string(), session.into());
    }

    /// Register the newly created window to the corresponding session in the AppState.
    pub fn window_created(&self, id: &str, session_id: &str) {
        let mut guard = self.window_to_session.lock().unwrap_or_else(|_e| {
            panic!("Main application state is poisoned. Cannot recover.");
        });
        let map = guard.deref_mut();
        if map.contains_key(id) {
            // TODO: This should be a normal error.
            panic!("Window already exists.");
        }
        map.insert(id.to_string(), session_id.to_string());
    }

    /// Get the ID of the session that owns given Window.
    pub fn get_session_id(&self, window: &Window) -> String {
        let guard = self.window_to_session.lock().unwrap_or_else(|_e| {
            panic!("Main application state is poisoned. Cannot recover.");
        });
        let map = guard.deref();
        map.get(window.label()).cloned().unwrap_or_else(|| {
            panic!("Unknown window label {}.", window.label());
        })
    }

    /// Get IDs of all windows corresponding to a given session.
    pub fn get_windows_for_session(&self, session_id: &str) -> Vec<String> {
        let guard = self.window_to_session.lock().unwrap_or_else(|_e| {
            panic!("Main application state is poisoned. Cannot recover.");
        });
        let map = guard.deref();

        map.iter()
            .filter_map(|(window, session)| {
                if session == session_id {
                    Some(window.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Emit state change events to all front-end windows corresponding to a given session.
    pub fn emit_to_session_windows(
        &self,
        app: &AeonApp,
        session_id: &str,
        state_change: StateChange,
    ) -> Result<(), DynError> {
        let window_ids = self.get_windows_for_session(session_id);
        if window_ids.is_empty() {
            warning!("No windows correspond to provided session `{session_id}`.")
        }
        if let Err(e) = app.tauri.emit_filter(AEON_VALUE, state_change.events, |w| {
            window_ids.contains(&w.label().to_string())
        }) {
            return AeonError::throw_with_source("Error sending state-change event.", e);
        }
        Ok(())
    }

    pub fn consume_event(
        &self,
        app: &AeonApp,
        session_id: &str,
        action: &UserAction,
    ) -> Result<(), DynError> {
        let mut guard = self
            .session_state
            .lock()
            .unwrap_or_else(|_e| panic!("Main application state is poisoned. Cannot recover."));
        let session = guard.deref_mut().get_mut(session_id).unwrap_or_else(|| {
            panic!("Unknown session id {}.", session_id);
        });
        let state_change = session.perform_action(action)?;
        self.emit_to_session_windows(app, session_id, state_change)
    }

    pub fn consume_message(
        &self,
        app: &AeonApp,
        session_to_id: &str,
        session_from_id: &str,
        message: &SessionMessage,
    ) -> Result<(), DynError> {
        let mut guard = self
            .session_state
            .lock()
            .unwrap_or_else(|_e| panic!("Main application state is poisoned. Cannot recover."));
        let session = guard.deref_mut().get_mut(session_to_id).unwrap_or_else(|| {
            panic!("Unknown session id {}.", session_to_id);
        });
        let (opt_response, opt_state_change) = session.process_message(message)?;

        // if message requires a response, trigger it
        if let Some(response_message) = opt_response {
            // swap from/to IDs
            let wrapped_message = AeonMessage {
                session_from: session_to_id.to_string(),
                session_to: session_from_id.to_string(),
                message: response_message.message,
            };
            let msg_serialized = serde_json::to_string(&wrapped_message)?;
            app.tauri.trigger_global(AEON_MESSAGE, Some(msg_serialized));
        }
        // if message requires a state change to be sent to frontend, emit it
        if let Some(state_change) = opt_state_change {
            return self.emit_to_session_windows(app, session_to_id, state_change);
        }
        Ok(())
    }

    pub fn refresh(
        &self,
        app: &AeonApp,
        session_id: &str,
        full_path: &[String],
    ) -> Result<(), DynError> {
        let mut guard = self
            .session_state
            .lock()
            .unwrap_or_else(|_e| panic!("Main application state is poisoned. Cannot recover."));
        let session = guard.deref_mut().get_mut(session_id).unwrap_or_else(|| {
            panic!("Unknown session id {}.", session_id);
        });
        let at_path = full_path.iter().map(|it| it.as_str()).collect::<Vec<_>>();
        let event = session.refresh(full_path, &at_path)?;
        let state_change = StateChange {
            events: vec![event],
        };
        self.emit_to_session_windows(app, session_id, state_change)
    }
}
