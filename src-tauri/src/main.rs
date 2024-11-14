// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use biodivine_sketchbook::app::event::{Event, SessionMessage, StateChange, UserAction};
use biodivine_sketchbook::app::event_wrappers::{AeonAction, AeonMessage, AeonRefresh};
use biodivine_sketchbook::app::state::analysis::AnalysisSession;
use biodivine_sketchbook::app::state::editor::EditorSession;
use biodivine_sketchbook::app::state::{AppState, DynSession};
use biodivine_sketchbook::app::{
    AeonApp, AEON_ACTION, AEON_MESSAGE, AEON_REFRESH, DEFAULT_SESSION_ID, DEFAULT_WINDOW_ID,
};
use biodivine_sketchbook::debug;
use chrono::prelude::*;
use tauri::{command, Manager, State, Window};

#[command]
fn get_session_id(window: Window, state: State<AppState>) -> String {
    state.get_session_id(&window)
}

fn main() {
    // Initialize empty app state.
    let state = AppState::default();

    let session: DynSession = Box::new(EditorSession::new(DEFAULT_SESSION_ID));
    state.session_created(DEFAULT_SESSION_ID, session);
    state.window_created(DEFAULT_WINDOW_ID, DEFAULT_SESSION_ID);

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

                // check for "new-session" events here
                // TODO: this part should be probably moved elsewhere, just a placeholder for now
                if action.events.len() == 1 && action.events[0].path == ["new-analysis-session"] {
                    // This `new-analysis-session` event comes from the Editor with the sketch that will be analyzed.
                    // Before starting the new analysis session, we run a consistency check on that sketch.
                    // If the check is successful, we continue creating the session. If not, user should be
                    // notified about the consistency issues, and we do not create the new session.

                    // Run this event that automatically fails and returns error if sketch is not consistent
                    let consistency_assert_event = UserAction {
                        events: vec![Event::build(&["sketch", "assert_consistency"], None)]
                    };
                    if let Err(e) = state.consume_event(&aeon, &session_id, &consistency_assert_event) {
                        // If sketch was inconsistent, lets first run this event that sends a summary of issues to the frontend.
                        // This event should not fail, it only updates frontend.
                        let consistency_check_event = UserAction {
                            events: vec![Event::build(&["sketch", "check_consistency"], None)]
                        };
                        state.consume_event(&aeon, &session_id, &consistency_check_event).unwrap();

                        // Now that user has all the details, lets just log the problem and send an error event to FE
                        debug!("Error while starting analysis workflow: `{}`.", e.to_string());
                        let message = "Sketch is not consistent. See detailed summary in the 'Consistency Check' section.";
                        // A crude way to escape the error message and wrap it in quotes.
                        let json_message: String = serde_json::Value::String(message.to_string()).to_string();
                        let state_change = StateChange {
                            events: vec![Event::build(&["error"], Some(&json_message))],
                        };
                        let res_emit =
                            state.emit_to_session_windows(&aeon, &session_id, state_change);
                        if let Err(e) = res_emit {
                            panic!("Event error failed to be sent: {:?}", e);
                        }
                    } else {
                        // If sketch is consistent, we are ready to create the new session. 

                        // prepare (timestamped) session and window instances for AppState
                        let time_now = Utc::now();
                        let timestamp = time_now.timestamp();
                        let new_session_id = format!("analysis-{timestamp}");
                        let new_window_id = format!("analysis-{timestamp}-window");
                        let new_session: DynSession = Box::new(AnalysisSession::new(&new_session_id));
                        state.session_created(&new_session_id, new_session);
                        state.window_created(&new_window_id, &new_session_id);

                        // send request message "from" the new analysis session to the editor session
                        // asking to transfer Sketch data
                        let message = SessionMessage {
                            message: Event::build(&["send_sketch"], None),
                        };
                        let res =
                            state.consume_message(&aeon, DEFAULT_SESSION_ID, &new_session_id, &message);
                        if let Err(e) = res {
                            panic!(
                                "Failed transferring sketch data from editor to analysis: {:?}",
                                e
                            );
                        }

                        // create a new window for the analysis session in tauri
                        let new_window = tauri::WindowBuilder::new(
                            &handle,
                            &new_window_id,
                            tauri::WindowUrl::App("src/html/analysis.html".into()),
                        )
                        .title(format!(
                            "Inference Workflow (started on {})",
                            time_now.to_rfc2822()
                        ))
                        .min_inner_size(800., 600.)
                        .inner_size(1000., 666.)
                        .build();

                        match new_window {
                            Ok(_) => debug!(
                                "New session `{new_session_id}` and window `{new_window_id}` created."
                            ),
                            Err(e) => panic!("Failed to create new window: {:?}", e),
                        }
                    }
                } else {
                    let result = state.consume_event(&aeon, &session_id, &action);
                    if let Err(e) = result {
                        // TODO: This is a temporary solution to propagate this kind of error message to frontend.
                        debug!("Error processing last event: `{}`.", e.to_string());
                        // A crude way to escape the error message and wrap it in quotes.
                        let json_message = serde_json::Value::String(e.to_string()).to_string();
                        let state_change = StateChange {
                            events: vec![Event::build(&["error"], Some(&json_message))],
                        };
                        let res_emit =
                            state.emit_to_session_windows(&aeon, &session_id, state_change);
                        if let Err(e) = res_emit {
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
            let aeon = aeon_original.clone();
            app.listen_global(AEON_MESSAGE, move |e| {
                let Some(payload) = e.payload() else {
                    // TODO: This should be an error.
                    panic!("No payload in backend message.");
                };
                debug!("Received backend message: `{}`.", payload);
                let event: AeonMessage = match serde_json::from_str::<AeonMessage>(payload) {
                    Ok(message) => message,
                    Err(e) => {
                        // TODO: This should be a normal error.
                        panic!("Payload deserialize error {:?}.", e);
                    }
                };
                let state = aeon.tauri.state::<AppState>();
                let session_from_id = event.session_from.clone();
                let session_to_id = event.session_to.clone();
                let message = SessionMessage {
                    message: event.message,
                };
                let result = state.consume_message(
                    &aeon,
                    session_to_id.as_str(),
                    session_from_id.as_str(),
                    &message,
                );
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
