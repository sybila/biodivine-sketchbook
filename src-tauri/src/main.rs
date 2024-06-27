// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aeon_sketchbook::app::event::{Event, UserAction};
use aeon_sketchbook::app::state::editor::EditorSession;
use aeon_sketchbook::app::state::{AppState, DynSession};
use aeon_sketchbook::app::{AeonApp, AEON_ACTION, AEON_REFRESH, AEON_VALUE};
use aeon_sketchbook::debug;
use serde::{Deserialize, Serialize};
use tauri::{command, Manager, State, Window};

#[command]
fn get_session_id(window: Window, state: State<AppState>) -> String {
    state.get_session_id(&window)
}

#[derive(Serialize, Deserialize)]
struct AeonAction {
    session: String,
    events: Vec<Event>,
}

#[derive(Serialize, Deserialize)]
struct AeonRefresh {
    session: String,
    path: Vec<String>,
}

fn main() {
    // Initialize empty app state.
    let state = AppState::default();
    let session: DynSession = Box::new(EditorSession::new("editor-1"));
    state.session_created("editor-1", session);
    state.window_created("editor", "editor-1");

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            let handle = app.handle();
            let aeon_original = AeonApp {
                tauri: handle.clone(),
            };
            let aeon = aeon_original.clone();
            app.listen_global(AEON_ACTION, move |e| {
                let Some(payload) = e.payload() else {
                    // TODO: This should be an error.
                    panic!("No payload in user action.");
                };
                debug!("Received user action: `{}`.", payload);
                let event: AeonAction = match serde_json::from_str::<AeonAction>(payload) {
                    Ok(action) => action,
                    Err(e) => {
                        // TODO: This should be a normal error.
                        panic!("Payload deserialize error {:?}.", e);
                    }
                };
                let state = aeon.tauri.state::<AppState>();
                let session_id = event.session.clone();
                let action = UserAction {
                    events: event.events,
                };
                let result = state.consume_event(&aeon, session_id.as_str(), &action);
                if let Err(e) = result {
                    // TODO: This should be a normal error.
                    //panic!("Event error: {:?}", e);

                    // TODO: This is only a temporary solution to propagate the error message to frontend.
                    debug!("Error processing last event: `{}`.", e.to_string());
                    // A crude way to escape the error message and wrap it in quotes.
                    let json_message = serde_json::Value::String(e.to_string()).to_string();
                    let state_change = Event::build(&["error"], Some(&json_message));
                    if aeon.tauri.emit_all(AEON_VALUE, vec![state_change]).is_err() {
                        panic!("Event error failed to be sent: {:?}", e);
                    }
                }
            });
            let aeon = aeon_original.clone();
            app.listen_global(AEON_REFRESH, move |e| {
                let Some(payload) = e.payload() else {
                    // TODO: This should be an error.
                    panic!("No payload in user action.");
                };
                debug!("Received user action: `{}`.", payload);
                let event: AeonRefresh = match serde_json::from_str::<AeonRefresh>(payload) {
                    Ok(action) => action,
                    Err(e) => {
                        // TODO: This should be a normal error.
                        panic!("Payload deserialize error {:?}.", e);
                    }
                };
                let state = aeon.tauri.state::<AppState>();
                let session_id = event.session.clone();
                let path = event.path;
                let result = state.refresh(&aeon, session_id.as_str(), &path);
                if let Err(e) = result {
                    // TODO: This should be a normal error.
                    panic!("Event error: {:?}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_session_id])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
