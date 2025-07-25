use crate::app::event::Event;
use crate::app::state::{Consumed, SessionState};
use crate::app::{AeonError, DynError};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Atomic state is a [SessionState] which holds exactly one value of a generic type `T`.
///
/// The generic type `T` needs to implement [Serialize] and [DeserializeOwned] to make it
/// serializable into the [Event] payload. Furthermore, we need [PartialEq] to check
/// if the value changed, and [Clone] to enable state replication.
///
/// Each [AtomicState] only consumes one type of event: the remaining `path` must be empty
/// and the event `payload` must deserialize into a valid `T` value.
#[derive(Clone, Debug)]
pub struct AtomicState<T>(T)
where
    T: Serialize + DeserializeOwned + PartialEq + Clone;

impl<T: Serialize + DeserializeOwned + PartialEq + Clone> From<T> for AtomicState<T> {
    fn from(value: T) -> Self {
        AtomicState(value)
    }
}

impl<T: Serialize + DeserializeOwned + PartialEq + Clone + Default> Default for AtomicState<T> {
    fn default() -> Self {
        AtomicState(T::default())
    }
}

impl<T: Serialize + DeserializeOwned + PartialEq + Clone> AtomicState<T> {
    pub fn value_string(&self) -> String {
        serde_json::to_string(&self.0).unwrap_or_else(|_e| {
            unreachable!("Value was received as payload but cannot be converted back?");
        })
    }
}

impl<T: Serialize + DeserializeOwned + PartialEq + Clone> SessionState for AtomicState<T> {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        if !at_path.is_empty() {
            let msg = format!("Atomic state cannot consume a path `{at_path:?}`.");
            return AeonError::throw(msg);
        }
        let Some(payload) = &event.payload else {
            return AeonError::throw("Missing payload for atomic state event.");
        };
        let Ok(payload) = serde_json::from_str(payload.as_str()) else {
            let msg = format!(
                "Cannot convert input `{}` to type `{}`.",
                payload,
                std::any::type_name::<T>()
            );
            return Ok(Consumed::InputError(Box::new(AeonError::new(msg, None))));
        };
        if self.0 == payload {
            return Ok(Consumed::NoChange);
        }
        let perform_event = event.clone();
        let mut reverse_event = event.clone();
        let old_value_str = self.value_string();
        reverse_event.payload = Some(old_value_str);
        self.0 = payload;

        Ok(Consumed::Reversible {
            state_change: perform_event.clone(),
            perform_reverse: (perform_event, reverse_event),
        })
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        if !at_path.is_empty() {
            let msg = format!("Atomic state cannot consume a path `{at_path:?}`.");
            return AeonError::throw(msg);
        }
        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(self.value_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::app::event::Event;
    use crate::app::state::_state_atomic::AtomicState;
    use crate::app::state::{Consumed, SessionState};

    #[test]
    fn test_atomic_state() {
        let mut state = AtomicState::from(3);

        let event_3 = Event::build(&["segment"], Some("3"));
        let event_4 = Event::build(&["segment"], Some("4"));
        let event_empty = Event::build(&["segment"], None);
        let event_invalid = Event::build(&["segment"], Some("abc"));

        let result = state.perform_event(&event_3, &[]).unwrap();
        assert!(matches!(result, Consumed::NoChange));

        let result = state.perform_event(&event_4, &[]).unwrap();
        assert!(matches!(result, Consumed::Reversible { .. }));

        let result = state.perform_event(&event_3, &["foo"]).unwrap_err();
        assert_eq!(
            "Atomic state cannot consume a path `[\"foo\"]`.",
            format!("{}", result)
        );
        let result = state.perform_event(&event_empty, &[]).unwrap_err();
        assert_eq!(
            "Missing payload for atomic state event.",
            format!("{}", result)
        );
        let result = state.perform_event(&event_invalid, &[]).unwrap();
        assert!(matches!(result, Consumed::InputError(..)));
    }
}
