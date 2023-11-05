use crate::app::event::UserAction;
use crate::app::state::{Consumed, DynSessionState, SessionState};
use crate::app::{AeonError, DynError};
use std::collections::HashMap;

/// A group of several dynamic state objects accessible using path segments.
///
/// It is generally assumed that the managed state objects are created statically at the
/// same time as [MapState], because we cannot create/remove states dynamically from this
/// map using just events (although you could in theory do it internally from Rust).
#[derive(Default)]
pub struct MapState {
    pub state: HashMap<String, DynSessionState>,
}

impl<'a> FromIterator<(&'a str, DynSessionState)> for MapState {
    fn from_iter<T: IntoIterator<Item = (&'a str, DynSessionState)>>(iter: T) -> Self {
        let map = HashMap::from_iter(iter.into_iter().map(|(k, v)| (k.to_string(), v)));
        MapState { state: map }
    }
}

impl SessionState for MapState {
    fn consume_event(&mut self, path: &[&str], action: &UserAction) -> Result<Consumed, DynError> {
        let Some(prefix) = path.first() else {
            return AeonError::throw("State map cannot consume an empty path.");
        };
        let Some(sub_state) = self.state.get_mut(*prefix) else {
            let msg = format!("Unknown path segment `{}`.", prefix);
            return AeonError::throw(msg);
        };
        sub_state.consume_event(&path[1..], action)
    }
}

#[cfg(test)]
mod tests {
    use crate::app::event::{Event, UserAction};
    use crate::app::state::_state_map::MapState;
    use crate::app::state::{AtomicState, Consumed, DynSessionState, SessionState};

    #[test]
    fn test_map_state() {
        let inner: Vec<(&str, DynSessionState)> = vec![
            ("state_1", Box::new(AtomicState::from(5i32))),
            ("state_2", Box::new(AtomicState::from("test".to_string()))),
            ("state_3", Box::new(AtomicState::from(123u64))),
        ];
        let mut state_map = MapState::from_iter(inner.into_iter());

        let event_3: UserAction = Event::build(&["state_1"], Some("3")).into();
        let event_str: UserAction = Event::build(&["state_2"], Some("test")).into();

        let result = state_map.consume_event(&["state_1"], &event_3).unwrap();
        assert!(matches!(result, Consumed::Reversible(..)));
        let result = state_map.consume_event(&["state_2"], &event_str).unwrap();
        assert!(matches!(result, Consumed::NoChange));

        let result = state_map.consume_event(&[], &event_3).unwrap_err();
        assert_eq!(
            "State map cannot consume an empty path.",
            format!("{}", result)
        );

        let result = state_map.consume_event(&["state_4"], &event_3).unwrap_err();
        assert_eq!("Unknown path segment `state_4`.", format!("{}", result));
    }
}
