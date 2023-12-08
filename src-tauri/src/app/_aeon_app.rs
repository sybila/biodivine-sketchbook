use tauri::AppHandle;

/// Serves as a global "application context" through which we can emit events or modify the
/// application state.
///
/// TODO: This isn't really implemented yet, it might never be? We'll see if we actually need it.
#[derive(Clone)]
pub struct AeonApp {
    pub tauri: AppHandle,
}
