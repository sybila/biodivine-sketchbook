use crate::app::event::{StateChange, UserAction};
use crate::app::state::{Consumed, SessionState};
use crate::app::{AeonError, DynError};
use std::str::FromStr;

/// Atomic state is a [SessionState] which holds exactly one value of a generic type `T`.
///
/// The generic type `T` needs to implement [FromStr] and [ToString] to make it serializable
/// into the [crate::app::event::Event] payload. Furthermore, we need [PartialEq] to check
/// if the value changed, and [Clone] to enable state replication.
///
/// Each [AtomicState] only consumes one type of event: the remaining `path` must be empty
/// and the event `payload` must deserialize into a valid `T` value.
#[derive(Clone, Debug)]
pub struct AtomicState<T>(T)
where
    T: FromStr + ToString + PartialEq + Clone;

impl<T: FromStr + ToString + PartialEq + Clone> From<T> for AtomicState<T> {
    fn from(value: T) -> Self {
        AtomicState(value)
    }
}

impl<T: FromStr + ToString + PartialEq + Clone + Default> Default for AtomicState<T> {
    fn default() -> Self {
        AtomicState(T::default())
    }
}

impl<T: FromStr + ToString + PartialEq + Clone> SessionState for AtomicState<T> {
    fn consume_event(&mut self, path: &[&str], action: &UserAction) -> Result<Consumed, DynError> {
        if !path.is_empty() {
            let msg = format!("Atomic state cannot consume a path `{:?}`.", path);
            return AeonError::throw(msg);
        }
        let Some(payload) = &action.event.payload else {
            return AeonError::throw("Missing payload for atomic state event.");
        };
        let Ok(payload) = T::from_str(payload) else {
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
        let perform_event = action.clone();
        let mut reverse_event = action.clone();
        reverse_event.event.payload = Some(self.0.to_string());
        self.0 = payload;

        Ok(Consumed::Reversible(
            StateChange::from(action.clone()),
            (perform_event, reverse_event),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::app::event::{Event, UserAction};
    use crate::app::state::_state_atomic::AtomicState;
    use crate::app::state::{Consumed, SessionState};

    #[test]
    fn test_atomic_state() {
        let mut state = AtomicState::from(3);

        let event_3: UserAction = Event::build(&["segment"], Some("3")).into();
        let event_4: UserAction = Event::build(&["segment"], Some("4")).into();
        let event_empty: UserAction = Event::build(&["segment"], None).into();
        let event_invalid: UserAction = Event::build(&["segment"], Some("abc")).into();

        let result = state.consume_event(&[], &event_3).unwrap();
        assert!(matches!(result, Consumed::NoChange));

        let result = state.consume_event(&[], &event_4).unwrap();
        assert!(matches!(result, Consumed::Reversible(..)));

        let result = state.consume_event(&["foo"], &event_3).unwrap_err();
        assert_eq!(
            "Atomic state cannot consume a path `[\"foo\"]`.",
            format!("{}", result)
        );
        let result = state.consume_event(&[], &event_empty).unwrap_err();
        assert_eq!(
            "Missing payload for atomic state event.",
            format!("{}", result)
        );
        let result = state.consume_event(&[], &event_invalid).unwrap();
        assert!(matches!(result, Consumed::InputError(..)));
    }
}
