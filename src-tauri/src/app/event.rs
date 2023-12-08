/// An [Event] object holds information about one particular state update.
///
/// It consists of a segmented `path` and an optional `payload`. The `payload` is expected
/// to be a JSON-encoded value.
///
/// Multiple events can be combined together to create one [UserAction] or [StateChange].
/// The purpose of this mechanism is to indicate that these events should be handled together,
/// as if they represented a single UI operations (e.g. they should only
/// hold one undo stack entry).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub path: Vec<String>,
    pub payload: Option<String>,
}

/// A [UserAction] is a collection of events that originate in one GUI action.
///
/// It does not necessarily need to be triggered by the user directly, but it is expected that
/// it somehow corresponds to a single action by the user to which the app should respond.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAction {
    pub events: Vec<Event>,
}

/// A [StateChange] is internally the same as [UserAction], but it represents a collection
/// of value updates that happened on the backend.
///
/// Typically, a [StateChange] is emitted as a result of a [UserAction]. But it can be also
/// triggered automatically, for example as part of a long-running computation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateChange {
    pub events: Vec<Event>,
}

impl Event {
    pub fn build(path: &[&str], payload: Option<&str>) -> Event {
        Event {
            path: path.iter().map(|it| it.to_string()).collect::<Vec<_>>(),
            payload: payload.map(|it| it.to_string()),
        }
    }

    /// An estimated amount of bytes consumed by the data stored in this [Event].
    pub fn byte_size(&self) -> usize {
        let path_len = self.path.iter().map(|it| it.len()).sum::<usize>();
        let payload_len = self.payload.as_ref().map(|it| it.len()).unwrap_or(0);
        path_len + payload_len
    }
}

impl UserAction {
    /// An estimated size of this [UserAction] in bytes.
    pub fn byte_size(&self) -> usize {
        self.events.iter().map(|it| it.byte_size()).sum::<usize>()
    }
}

impl From<UserAction> for StateChange {
    fn from(value: UserAction) -> Self {
        StateChange {
            events: value.events,
        }
    }
}

impl From<StateChange> for UserAction {
    fn from(value: StateChange) -> Self {
        UserAction {
            events: value.events,
        }
    }
}

impl From<Event> for StateChange {
    fn from(value: Event) -> Self {
        StateChange {
            events: vec![value],
        }
    }
}

impl From<Event> for UserAction {
    fn from(value: Event) -> Self {
        UserAction {
            events: vec![value],
        }
    }
}
