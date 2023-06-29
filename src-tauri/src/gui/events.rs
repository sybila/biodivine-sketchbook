/// At this point, we don't really know what types of GUI events we'll be sending,
/// so it's just a generic collection of values which tells us:
///  - What's going on (`action`);
///  - What component is affected (`path`);
///  - And whether there is any extra data (`payload`);
///
/// In the future, we might want to instead have an enum of a few common even types, plus
/// a very generic "any" event like this.
///
/// TODO:
///     - Add a nicer debug format.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct GuiEvent {
    path: Vec<String>,
    action: String,
    payload: Option<String>,
}

impl GuiEvent {
    pub fn with_action(path: &[String], action: &str) -> GuiEvent {
        GuiEvent {
            path: Vec::from_iter(path.iter().cloned()),
            action: action.to_string(),
            payload: None,
        }
    }

    pub fn with_action_and_payload(path: &[String], action: &str, payload: &str) -> GuiEvent {
        GuiEvent {
            path: Vec::from_iter(path.iter().cloned()),
            action: action.to_string(),
            payload: Some(payload.to_string()),
        }
    }

    /// An estimated size of the payload in this event, or zero if there is no payload.
    ///
    /// Note that this is not guaranteed to be the exact size. Rather, it is only an estimate
    /// to determine how costly it is to store a particular collection of events in memory.
    pub fn payload_size(&self) -> usize {
        self.payload.as_ref().map(|it| it.len()).unwrap_or(0)
    }

    pub fn action(&self) -> &str {
        self.action.as_str()
    }
}
