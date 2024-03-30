use crate::app::state::{Consumed, SessionState};

/// **(internal)** Tests for the event-based API of `ModelState`.
mod _model;
/// **(internal)** Tests for the event-based API of `ObservationManager`.
mod _observations;

/// Given a state of a manager class *after* a particular event is performed (`state_after_event`),
/// check that by performing a reverse event, we get precisely the original state (`orig_state`).
///
/// This function can be used to test events for all manager classes, such as `ModelState`,
/// `ObservationManager`, or `PropertyManager`.
///
/// - `result` is the result of the original event (carrying the reverse variant)
/// - `at_path` is the relative path for the reverse event
fn check_reverse<T: SessionState + std::fmt::Debug + PartialEq>(
    state_after_event: &mut T,
    orig_state: &T,
    result: Consumed,
    at_path: &[&str],
) {
    match result {
        Consumed::Reversible {
            perform_reverse: (_, reverse),
            ..
        } => {
            state_after_event.perform_event(&reverse, &at_path).unwrap();
            assert_eq!(state_after_event, orig_state);
        }
        _ => panic!(),
    }
}
