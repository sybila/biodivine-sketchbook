use crate::app::event::Event;
use crate::app::state::Consumed;
use crate::app::DynError;
use crate::sketchbook::JsonSerde;
use serde::Serialize;

/// Shorthand to create a `Consumed::Reversible` instance given all the components.
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

/// Shorthand to create a "refresh" event for a list of items.
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

/// Shorthand to create a "state-change" event given a path and potential payload.
/// Payload can be any struct that implements `JsonSerde`.
pub(super) fn make_state_change<'a, T>(path: &[&str], payload: &T) -> Event
where
    T: JsonSerde<'a>,
{
    Event::build(path, Some(&payload.to_json_str()))
}

/// Prepare "state-change" event for the `model` component of the `sketch`, given
/// `at_path` - a path suffix used at the model level, and potential payload.
/// Payload can be any struct that implements `JsonSerde`.
pub(super) fn mk_model_state_change<'a, T: JsonSerde<'a>>(at_path: &[&str], payload: &T) -> Event {
    let mut full_path = vec!["sketch", "model"];
    full_path.extend_from_slice(at_path);
    make_state_change(&full_path, payload)
}

/// Prepare "state-change" event for the `observations` component of the `sketch`, given
/// `at_path` - a path suffix used at the observations level, and potential payload.
/// Payload can be any struct that implements `JsonSerde`.
pub(super) fn mk_obs_state_change<'a, T: JsonSerde<'a>>(at_path: &[&str], payload: &T) -> Event {
    let mut full_path = vec!["sketch", "observations"];
    full_path.extend_from_slice(at_path);
    make_state_change(&full_path, payload)
}

/// Prepare "state-change" event for the `properties` component of the `sketch`, given
/// `at_path` - a path suffix used at the properties level, and potential payload.
/// Payload can be any struct that implements `JsonSerde`.
pub(super) fn _mk_prop_state_change<'a, T: JsonSerde<'a>>(at_path: &[&str], payload: &T) -> Event {
    let mut full_path = vec!["sketch", "properties"];
    full_path.extend_from_slice(at_path);
    make_state_change(&full_path, payload)
}

/// Prepare event for the `model` component of the `sketch`, given `at_path` - a path suffix
/// used at the model level, and a `payload`.
pub(super) fn mk_model_event(at_path: &[&str], payload: Option<&str>) -> Event {
    let mut full_path = vec!["sketch", "model"];
    full_path.extend_from_slice(at_path);
    Event::build(&full_path, payload)
}

/// Prepare event for the `observations` component of the `sketch`, given `at_path` - a path suffix
/// used at the observations manager level, and a `payload`.
pub(super) fn mk_obs_event(at_path: &[&str], payload: Option<&str>) -> Event {
    let mut full_path = vec!["sketch", "observations"];
    full_path.extend_from_slice(at_path);
    Event::build(&full_path, payload)
}

/// Prepare event for the `properties` component of the `sketch`, given `at_path` - a path suffix
/// used at the property manager level, and a `payload`.
pub(super) fn _mk_prop_event(at_path: &[&str], payload: Option<&str>) -> Event {
    let mut full_path = vec!["sketch", "properties"];
    full_path.extend_from_slice(at_path);
    Event::build(&full_path, payload)
}
