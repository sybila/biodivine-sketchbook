use crate::app::event::Event;
use serde::{Deserialize, Serialize};

/// Struct to wrap "user action" ([crate::app::event::UserAction]) events sent from
/// front-end to a particular session.
#[derive(Serialize, Deserialize)]
pub struct AeonAction {
    pub session: String,
    pub events: Vec<Event>,
}

/// Struct to wrap "refresh" requests sent from front-end to a particular session.
#[derive(Serialize, Deserialize)]
pub struct AeonRefresh {
    pub session: String,
    pub path: Vec<String>,
}

/// Struct to wrap "backend message" ([crate::app::event::SessionMessage]) events exchanged
/// between particular sessions on back-end.
#[derive(Serialize, Deserialize)]
pub struct AeonMessage {
    pub session_from: String,
    pub session_to: String,
    pub message: Event,
}
