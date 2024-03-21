use crate::app::event::Event;
use crate::app::state::Consumed;
use crate::app::DynError;
use serde::Serialize;
use std::fmt::Display;

/// Shorthand to create a `Consumed::Reversible`.
pub(super) fn make_reversible(
    state_change: Event,
    original_event: &Event,
    reverse_event: Event,
) -> Consumed {
    Consumed::Reversible {
        state_change,
        perform_reverse: (original_event.clone(), reverse_event),
    }
}

pub(super) fn make_refresh_event<T>(
    full_path: &[String],
    item_list: Vec<T>,
) -> Result<Event, DynError>
where
    T: Serialize,
{
    Ok(Event {
        path: full_path.to_vec(),
        payload: Some(serde_json::to_string(&item_list)?),
    })
}

/// Shorthand to create a state-change event.
/// Payload can be any struct that can be stringified (especially any class defined
/// in [crate::sketchbook::data_structs]).
pub(super) fn make_state_change<T>(path: &[&str], payload: &T) -> Event
where
    T: Display,
{
    Event::build(path, Some(&payload.to_string()))
}
