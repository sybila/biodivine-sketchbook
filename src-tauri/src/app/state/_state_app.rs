use crate::app::event::UserAction;
use crate::app::state::{Consumed, DynSessionState, MapState, SessionState};
use crate::app::{AeonApp, AeonError, DynError, UndoStack, EVENT_STATE_CHANGE};
use crate::debug;
use std::ops::DerefMut;
use std::sync::Mutex;
use tauri::Manager;

/// [AppState] is a special wrapper around [MapState] which implements the "top level"
/// container for the whole application state.
///
/// Specifically, it ensures:
///  - Synchronization by locking the app state.
///  - Ability to add/remove windows as they are opened/closed.
///  - Error handling and retransmission of events.
///
/// As such, [AppState] does not actually implement the [SessionState] trait.
pub struct AppState {
    state: Mutex<(MapState, UndoStack)>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            state: Mutex::new((MapState::default(), UndoStack::default())),
        }
    }
}

impl AppState {
    pub fn window_created(&self, label: &str, state: impl Into<DynSessionState>) {
        let mut guard = self.state.lock().unwrap_or_else(|_e| {
            panic!("Main application state is poisoned. Cannot recover.");
        });
        let (windows, _stack) = guard.deref_mut();
        if windows.state.contains_key(label) {
            // TODO: This should be a normal error.
            panic!("Window already exists");
        }
        windows.state.insert(label.to_string(), state.into());
    }

    pub fn undo(&self, app: &AeonApp) -> Result<(), DynError> {
        let mut guard = self
            .state
            .lock()
            .unwrap_or_else(|_e| panic!("Main application state is poisoned. Cannot recover."));
        let (windows, stack) = guard.deref_mut();
        let mut event = match stack.undo_action() {
            Some(reverse) => reverse,
            None => {
                debug!("Cannot undo.");
                return Ok(());
            }
        };
        let state_change = loop {
            let path = event
                .event
                .full_path
                .iter()
                .map(|it| it.as_str())
                .collect::<Vec<_>>();
            let result = windows.consume_event(&path, &event);
            match result {
                Err(error) => {
                    debug!("Event error: `{:?}`.", error);
                    return Err(error);
                }
                Ok(Consumed::NoChange) => {
                    debug!("No change.");
                    return Ok(());
                }
                // TODO: We should treat this error differently.
                Ok(Consumed::InputError(error)) => {
                    debug!("User input error: `{:?}`.", error);
                    return Err(error);
                }
                Ok(Consumed::Irreversible(_)) => {
                    // TODO: This probably shouldn't happen here.
                    panic!("Irreversible action as a result of undo.")
                }
                Ok(Consumed::Reversible(state, _)) => {
                    debug!("Reversible state change: `{:?}`.", state);
                    break state;
                }
                Ok(Consumed::Restart(action)) => {
                    event = action;
                }
            }
        };
        if let Err(e) = app.tauri.emit_all(EVENT_STATE_CHANGE, state_change.event) {
            return AeonError::throw_with_source("Error sending state event.", e);
        }
        Ok(())
    }

    pub fn redo(&self, app: &AeonApp) -> Result<(), DynError> {
        let mut guard = self
            .state
            .lock()
            .unwrap_or_else(|_e| panic!("Main application state is poisoned. Cannot recover."));
        let (windows, stack) = guard.deref_mut();
        let mut event = match stack.redo_action() {
            Some(perform) => perform,
            None => {
                debug!("Cannot redo.");
                return Ok(());
            }
        };
        let state_change = loop {
            let path = event
                .event
                .full_path
                .iter()
                .map(|it| it.as_str())
                .collect::<Vec<_>>();
            let result = windows.consume_event(&path, &event);
            match result {
                Err(error) => {
                    debug!("Event error: `{:?}`.", error);
                    return Err(error);
                }
                Ok(Consumed::NoChange) => {
                    debug!("No change.");
                    return Ok(());
                }
                // TODO: We should treat this error differently.
                Ok(Consumed::InputError(error)) => {
                    debug!("User input error: `{:?}`.", error);
                    return Err(error);
                }
                Ok(Consumed::Irreversible(_)) => {
                    // TODO: This probably shouldn't happen here.
                    panic!("Irreversible action as a result of redo.")
                }
                Ok(Consumed::Reversible(state, _)) => {
                    debug!("Reversible state change: `{:?}`.", state);
                    break state;
                }
                Ok(Consumed::Restart(action)) => {
                    event = action;
                }
            }
        };
        if let Err(e) = app.tauri.emit_all(EVENT_STATE_CHANGE, state_change.event) {
            return AeonError::throw_with_source("Error sending state event.", e);
        }
        Ok(())
    }

    pub fn consume_event(&self, app: &AeonApp, mut event: UserAction) -> Result<(), DynError> {
        let mut guard = self
            .state
            .lock()
            .unwrap_or_else(|_e| panic!("Main application state is poisoned. Cannot recover."));
        let (windows, stack) = guard.deref_mut();
        let state_change = loop {
            let path = event
                .event
                .full_path
                .iter()
                .map(|it| it.as_str())
                .collect::<Vec<_>>();
            let result = windows.consume_event(&path, &event);
            match result {
                Err(error) => {
                    debug!("Event error: `{:?}`.", error);
                    return Err(error);
                }
                Ok(Consumed::NoChange) => {
                    debug!("No change.");
                    return Ok(());
                }
                // TODO: We should treat this error differently.
                Ok(Consumed::InputError(error)) => {
                    debug!("User input error: `{:?}`.", error);
                    return Err(error);
                }
                Ok(Consumed::Irreversible(state)) => {
                    debug!("Irreversible state change: `{:?}`.", state);
                    stack.clear();
                    break state;
                }
                Ok(Consumed::Reversible(state, (perform, reverse))) => {
                    debug!("Reversible state change: `{:?}`.", state);
                    if !stack.do_action(perform, reverse) {
                        // TODO:
                        //  This is a warning, because the state has been applied at
                        //  this point, but we should think a bit more about how this
                        //  should be ideally handled.
                        stack.clear();
                    }
                    break state;
                }
                Ok(Consumed::Restart(action)) => {
                    event = action;
                }
            }
        };
        if let Err(e) = app.tauri.emit_all(EVENT_STATE_CHANGE, state_change.event) {
            return AeonError::throw_with_source("Error sending state event.", e);
        }
        Ok(())
    }
}
