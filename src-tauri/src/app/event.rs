/// An [Event] object holds information about one particular state update.
///
/// It consists of a segmented `path` and an optional `payload`. Under normal circumstances,
/// the payload is expected to be either a single value (e.g. a string encoding of a single
/// integer), or a JSON object (encoding multiple values).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub full_path: Vec<String>,
    pub payload: Option<String>,
}

/// A [UserAction] is a type of event that (in some sense) originates in the GUI.
///
/// It does not necessarily need to be triggered by the user directly, but it is expected that
/// it somehow corresponds to some action by the user to which the app should respond.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAction {
    pub event: Event,
}

/// A [StateChange] is a type of event that (in some sense) originates in the backend.
///
/// Typically, a state change is triggered once a [UserAction] is handled by the application
/// such that the state of the application changed. However, it can be also triggered
/// automatically, e.g. by a long-running computation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateChange {
    pub event: Event,
}

impl Event {
    pub fn build(path: &[&str], payload: Option<&str>) -> Event {
        Event {
            full_path: path.iter().map(|it| it.to_string()).collect::<Vec<_>>(),
            payload: payload.map(|it| it.to_string()),
        }
    }

    /// An estimated size of the `payload` in this event, or zero if there is no `payload`.
    ///
    /// Note that this is not guaranteed to be the exact size. Rather, it is only an estimate
    /// to determine how costly it is to store a particular collection of events in memory.
    pub fn payload_size(&self) -> usize {
        self.payload.as_ref().map(|it| it.len()).unwrap_or(0)
    }
}

impl From<UserAction> for StateChange {
    fn from(value: UserAction) -> Self {
        StateChange { event: value.event }
    }
}

impl From<StateChange> for UserAction {
    fn from(value: StateChange) -> Self {
        UserAction { event: value.event }
    }
}

impl From<Event> for StateChange {
    fn from(value: Event) -> Self {
        StateChange { event: value }
    }
}

impl From<Event> for UserAction {
    fn from(value: Event) -> Self {
        UserAction { event: value }
    }
}
