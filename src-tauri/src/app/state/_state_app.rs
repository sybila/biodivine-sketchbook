use crate::app::event::UserAction;
use crate::app::state::DynSession;
use crate::app::{AeonApp, AeonError, DynError, AEON_VALUE};
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
/// As such, [AppState] does not actually implement the [SessionState] trait.
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

    pub fn get_session_id(&self, window: &Window) -> String {
        let guard = self.window_to_session.lock().unwrap_or_else(|_e| {
            panic!("Main application state is poisoned. Cannot recover.");
        });
        let map = guard.deref();
        map.get(window.label()).cloned().unwrap_or_else(|| {
            panic!("Unknown window label {}.", window.label());
        })
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
        if let Err(e) = app.tauri.emit_all(AEON_VALUE, state_change.events) {
            return AeonError::throw_with_source("Error sending state event.", e);
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
        let state_change = session.refresh(full_path, &at_path)?;
        if let Err(e) = app.tauri.emit_all(AEON_VALUE, vec![state_change]) {
            return AeonError::throw_with_source("Error sending state event.", e);
        }

        Ok(())
    }
}
