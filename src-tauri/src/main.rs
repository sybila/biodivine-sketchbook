// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aeon_sketchbook::app::event::{Event, UserAction};
use aeon_sketchbook::app::state::{AppState, AtomicState, DynSessionState, MapState};
use aeon_sketchbook::app::{AeonApp, EVENT_USER_ACTION};
use aeon_sketchbook::debug;
use tauri::Manager;

fn main() {
    let editor_state: Vec<(&str, DynSessionState)> = vec![
        ("counter", Box::new(AtomicState::from(0))),
        ("text", Box::new(AtomicState::from("".to_string()))),
    ];
    let editor_state: DynSessionState = Box::new(MapState::from_iter(editor_state));
    let state = AppState::default();
    state.window_created("editor", editor_state);
    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            let handle = app.handle();
            let aeon_original = AeonApp {
                tauri: handle.clone(),
            };
            let aeon = aeon_original.clone();
            app.listen_global(EVENT_USER_ACTION, move |e| {
                let Some(payload) = e.payload() else {
                    // TODO: This should be an error.
                    panic!("No payload in user action.");
                };
                debug!("Received user action: `{}`.", payload);
                let event: UserAction = match serde_json::from_str::<Event>(payload) {
                    Ok(event) => event.into(),
                    Err(e) => {
                        // TODO: This should be a normal error.
                        panic!("Payload deserialize error {:?}.", e);
                    }
                };
                let state = aeon.tauri.state::<AppState>();
                let result = state.consume_event(&aeon, event);
                if let Err(e) = result {
                    // TODO: This should be a normal error.
                    panic!("Event error: {:?}", e);
                }
            });
            let aeon = aeon_original.clone();
            app.listen_global("undo", move |_e| {
                let state = aeon.tauri.state::<AppState>();
                if let Err(e) = state.undo(&aeon) {
                    // TODO: This should be a normal error.
                    panic!("Undo error: {:?}", e);
                }
            });
            let aeon = aeon_original.clone();
            app.listen_global("redo", move |_e| {
                let state = aeon.tauri.state::<AppState>();
                if let Err(e) = state.redo(&aeon) {
                    // TODO: This should be a normal error.
                    panic!("Redo error: {:?}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
