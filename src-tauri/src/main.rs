// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aeon_sketchbook::app::event::{Event, UserAction};
use aeon_sketchbook::app::state::analysis::AnalysisSession;
use aeon_sketchbook::app::state::editor::EditorSession;
use aeon_sketchbook::app::state::{AppState, DynSession};
use aeon_sketchbook::app::{AeonApp, AEON_ACTION, AEON_REFRESH, AEON_VALUE};
use aeon_sketchbook::debug;
use aeon_sketchbook::sketchbook::data_structs::SketchData;
use aeon_sketchbook::sketchbook::JsonSerde;
use aeon_sketchbook::sketchbook::Sketch;
use chrono::prelude::*;
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

                // TODO: this part should be probably moved elsewhere, just a placeholder for now
                // check for "new-session" events here
                if action.events.len() == 1 && action.events[0].path == ["new-analysis-session"] {
                    let payload = action.events[0]
                        .payload
                        .clone()
                        .ok_or(
                            "This `new-analysis-session` event cannot carry empty payload."
                                .to_string(),
                        )
                        .unwrap();
                    let sketch_data = SketchData::from_json_str(&payload).unwrap();
                    let sketch = Sketch::new_from_sketch_data(&sketch_data).unwrap();

                    let time_now = Utc::now();
                    let timestamp = time_now.timestamp();
                    let new_session_id = format!("analysis-{timestamp}");
                    let new_window_id = format!("analysis-{timestamp}-window");
                    let new_session: DynSession =
                        Box::new(AnalysisSession::new(&new_session_id, sketch));
                    state.session_created(&new_session_id, new_session);
                    state.window_created(&new_window_id, &new_session_id);

                    // Create a new window for the analysis session
                    let new_window = tauri::WindowBuilder::new(
                        &handle,
                        &new_window_id, // The unique window label
                        tauri::WindowUrl::App("src/html/analysis.html".into()), // The URL or path to the HTML file
                    )
                    .title(format!(
                        "Inference Workflow (opened on {})",
                        time_now.to_rfc2822()
                    ))
                    .build();

                    match new_window {
                        Ok(_) => debug!(
                            "New session `{new_session_id}` and window `{new_window_id}` created."
                        ),
                        Err(e) => panic!("Failed to create new window: {:?}", e),
                    }
                } else {
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
