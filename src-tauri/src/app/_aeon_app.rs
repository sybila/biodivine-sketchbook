use tauri::AppHandle;

/// Serves as a global "application context" through which we can emit events or modify the
/// application state.
#[derive(Clone)]
pub struct AeonApp {
    pub tauri: AppHandle,
}
