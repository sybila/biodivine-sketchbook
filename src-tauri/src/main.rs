// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use biodivine_sketchbook::app::event::{Event, SessionMessage, StateChange, UserAction};
use biodivine_sketchbook::app::event_wrappers::{AeonAction, AeonMessage, AeonRefresh};
use biodivine_sketchbook::app::state::editor::EditorSession;
use biodivine_sketchbook::app::state::inference::InferenceSession;
use biodivine_sketchbook::app::state::{AppState, DynSession};
use biodivine_sketchbook::app::{
    AeonApp, AEON_ACTION, AEON_MESSAGE, AEON_REFRESH, DEFAULT_SESSION_ID, DEFAULT_WINDOW_ID,
};
use biodivine_sketchbook::{debug, error};
use chrono::prelude::*;
use std::panic;
use std::process;
use tauri::api::dialog::blocking::MessageDialogBuilder;
use tauri::{command, AppHandle, Manager, State, Window};

#[command]
fn get_session_id(window: Window, state: State<AppState>) -> String {
    state.get_session_id(&window)
}

/// Wrapper to escape a string message and wrap it in quotes (in a crude way).
fn escape_string_json(message: &str) -> String {
    serde_json::Value::String(message.to_string()).to_string()
}

/// Extract payload (as string) from the Tauri event. If there is no payload,
/// log error and panic, since this is an critical issue.
fn get_payload_or_panic(event: tauri::Event, event_type: &str) -> String {
    let Some(payload) = event.payload() else {
        let message = format!("No payload in {event_type}.");
        error!("{message}");
        // TODO: think about whether this could be a normal error
        panic!("{message}");
    };
    payload.to_string()
}

/// Handle `Result` errors with logging and optional panic.
///
/// Panics if `should_panic` is true and the `result` is an `Err`.
/// Logged message contains the provided `context` and the given error.
fn handle_result<T, E: std::fmt::Debug>(result: Result<T, E>, context: &str, should_panic: bool) {
    if let Err(e) = result {
        let message = format!("{}: {:?}", context, e);
        error!("{message}");
        if should_panic {
            panic!("{message}");
        }
    }
}

/// Emit error to the session's frontend using JSON payload via a prepared event.
///
/// Panics if the error cant be send due to some internal Tauri issues.
/// In future, we can modularize different errors.
fn emit_error(state: &AppState, session_id: &str, aeon: &AeonApp, error_message: &str) {
    let json_message = escape_string_json(error_message); // Excape the message in quotes.
    let state_change = StateChange {
        events: vec![Event::build(&["error"], Some(&json_message))],
    };
    let res_emit = state.emit_to_session_windows(aeon, session_id, state_change);

    let context_msg = format!("Unable to emit error to frontend of session `{session_id}`");
    handle_result(res_emit, &context_msg, true);
}

/// Handle the set up of a new inference session (initiated at the editor session `editor_session_id`).
///
/// Before starting the new inference session, run a consistency check on the sketch data.
/// If the check is successful, continue creating the session. If not, user should be
/// notified about the consistency issues, and we do not create the new session.
fn handle_new_inference_session(
    handle: &AppHandle,
    state: &State<'_, AppState>,
    aeon: &AeonApp,
    editor_session_id: &str,
) {
    // Run this event to check the consistency
    // The event would return error if there are inconsistencies in the sketch
    let consistency_assert_event = UserAction {
        events: vec![Event::build(&["sketch", "assert_consistency"], None)],
    };
    let consistency_res = state.consume_event(aeon, editor_session_id, &consistency_assert_event);
    if let Err(e) = consistency_res {
        // If sketch was inconsistent, lets first send this event with a summary of issues to the frontend
        let consistency_check_event = UserAction {
            events: vec![Event::build(&["sketch", "check_consistency"], None)],
        };
        // This event processing can't return error (it only collects error messages into its payload)
        state
            .consume_event(aeon, editor_session_id, &consistency_check_event)
            .unwrap();

        // Now that user has all the details, lets just log the problem and send a proper error event to FE
        debug!(
            "Could not start inference session due to: `{}`.",
            e.to_string()
        );
        let message =
            "Sketch is not consistent. See detailed summary in the 'Consistency Check' section.";
        emit_error(state, editor_session_id, aeon, message);
    } else {
        // If sketch is consistent, we are ready to create a new session.
        // 1) prepare session and window IDs, and save them to AppState
        let time_now = Utc::now();
        let timestamp = time_now.timestamp();
        let new_session_id = format!("inference-{timestamp}");
        let new_window_id = format!("inference-{timestamp}-window");
        let new_session: DynSession = Box::new(InferenceSession::new(&new_session_id));
        state.session_created(&new_session_id, new_session);
        state.window_created(&new_window_id, &new_session_id);

        // 2) send a request message "from" the new inference session to the editor session
        //    asking to transfer Sketch data
        let message = SessionMessage {
            message: Event::build(&["send_sketch"], None),
        };
        let res = state.consume_message(aeon, DEFAULT_SESSION_ID, &new_session_id, &message);
        let error_msg = "Failed transferring sketch data from editor to inference session.";
        handle_result(res, error_msg, true);

        // 3) create a new frontend window for the inference session in tauri
        let title = format!("Inference session (started on {})", time_now.to_rfc2822());
        let new_window = tauri::WindowBuilder::new(
            handle,
            &new_window_id,
            tauri::WindowUrl::App("src/html/analysis.html".into()),
        )
        .title(title)
        .min_inner_size(800., 600.)
        .inner_size(1000., 666.)
        .build();

        match new_window {
            Ok(_) => debug!("New session `{new_session_id}` and window `{new_window_id}` created."),
            Err(e) => panic!("Failed to create new window: {:?}", e),
        }
    }
}

/// Deserialize the Tauri event's payload string into the target type.
///
/// Panics if deserialization fails, logging the error with context.
fn deserialize_payload<T: serde::de::DeserializeOwned>(payload: &str, event_type: &str) -> T {
    serde_json::from_str::<T>(payload).unwrap_or_else(|e| {
        // TODO: think about whether this could be a normal error
        let message = format!("Failed to deserialize {event_type} payload: {:?}", e);
        error!("{message}");
        panic!("{message}");
    })
}

/// Handles `AEON_ACTION` events.
fn process_aeon_action_event(payload: &str, aeon: &AeonApp, handle: &AppHandle) {
    debug!("Received user action: `{}`.", payload);
    let event: AeonAction = deserialize_payload(payload, AEON_ACTION);
    let state = aeon.tauri.state::<AppState>();
    let session_id = event.session.clone();
    let action = UserAction {
        events: event.events,
    };

    // check for "new-session" events here
    if action.events.len() == 1 && action.events[0].path == ["new-inference-session"] {
        // This `new-inference-session` event comes from the Editor with the sketch that will be analyzed.
        handle_new_inference_session(handle, &state, aeon, &session_id);
    } else {
        let result = state.consume_event(aeon, &session_id, &action);
        if let Err(e) = result {
            // there may be better way to propagate the message to frontend
            let error_message = e.to_string();
            debug!("Error processing last event: `{}`.", error_message);
            emit_error(&state, &session_id, aeon, &error_message);
        }
    }
}

/// Handles `AEON_REFRESH` events.
fn process_aeon_refresh_event(payload: &str, aeon: &AeonApp) {
    debug!("Received user refresh action: `{}`.", payload);
    let event: AeonRefresh = deserialize_payload(payload, AEON_REFRESH);
    let state = aeon.tauri.state::<AppState>();
    let session_id = event.session.clone();
    let path = event.path;
    let result = state.refresh(aeon, session_id.as_str(), &path);
    handle_result(result, "Internal refresh event error", true);
}

/// Handles `AEON_MESSAGE` events.
fn process_aeon_message_event(payload: &str, aeon: &AeonApp) {
    debug!("Received backend message: `{}`.", payload);
    let event: AeonMessage = deserialize_payload(payload, AEON_REFRESH);
    let state = aeon.tauri.state::<AppState>();
    let session_from_id = event.session_from.clone();
    let session_to_id = event.session_to.clone();
    let message = SessionMessage {
        message: event.message,
    };
    let result = state.consume_message(
        aeon,
        session_to_id.as_str(),
        session_from_id.as_str(),
        &message,
    );
    handle_result(result, "Internal backend event error", true);
}

fn main() {
    // Initialize empty app state.
    let state = AppState::default();

    // Register the intial session.
    let session: DynSession = Box::new(EditorSession::new(DEFAULT_SESSION_ID));
    state.session_created(DEFAULT_SESSION_ID, session);
    state.window_created(DEFAULT_WINDOW_ID, DEFAULT_SESSION_ID);

    // Build the app, registering the state and all Tauri event handlers.
    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            let orig_hook = panic::take_hook();
            panic::set_hook(Box::new(move |panic_info| {
                // Show backtrace to the user (this does not include line numbers, but
                // should at least be somewhat informative).
                let backtrace = std::backtrace::Backtrace::force_capture();
                MessageDialogBuilder::new("Unexpected error", format!("{}", backtrace)).show();
                // invoke the default handler and exit the process
                orig_hook(panic_info);
                process::exit(1);
            }));

            let handle = app.handle();
            let aeon_original = AeonApp {
                tauri: handle.clone(),
            };
            let aeon = aeon_original.clone();
            app.listen_global(AEON_ACTION, move |e| {
                let payload = get_payload_or_panic(e, AEON_ACTION);
                process_aeon_action_event(&payload, &aeon, &handle)
            });
            let aeon = aeon_original.clone();
            app.listen_global(AEON_REFRESH, move |e| {
                let payload = get_payload_or_panic(e, AEON_REFRESH);
                process_aeon_refresh_event(&payload, &aeon)
            });
            let aeon = aeon_original.clone();
            app.listen_global(AEON_MESSAGE, move |e| {
                let payload = get_payload_or_panic(e, AEON_MESSAGE);
                process_aeon_message_event(&payload, &aeon)
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_session_id])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
