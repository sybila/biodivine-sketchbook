use crate::app::event::{StateChange, UserAction};
use crate::app::DynError;

/// A [Consumed] object describes possible outcomes of trying to consume a [UserAction]
/// by the [AppState].
#[derive(Debug)]
pub enum Consumed {
    /// Action was successfully consumed, resulting in the given [StateChange].
    ///
    /// Furthermore, the action is reversible, meaning it can be saved to the back-stack
    /// using the provided pair of `(perform, reverse)` actions. Note that the `perform`
    /// action can be a copy of the original user action, but it can be also a different event,
    /// for example if the state object performed some automatic value normalization.
    ///
    /// Note that this does not *guarantee* that the action will be saved to the back stack.
    /// If the payloads for the `(perform, reverse)` actions are too large, [AppState] can
    /// still refuse to save it, in which case the stack will be simply reset to empty.
    Reversible(StateChange, (UserAction, UserAction)),

    /// Action was successfully consumed, resulting in the given [StateChange].
    ///
    /// However, the action is irreversible, meaning the back-stack needs to be reset.
    Irreversible(StateChange),

    /// Action cannot be consumed at its intended path and should be re-emitted as
    /// the freshly constructed [UserAction].
    Restart(UserAction),

    /// The action was consumed, but the provided user input is invalid and cannot be applied.
    ///
    /// Note that this should only be used when the *user input* is wrong. If some other part
    /// of the action is wrong (e.g. unknown path, missing fields in payload), the action cannot
    /// be consumed!
    InputError(DynError),

    /// The action was consumed, but the application state did not change.
    NoChange,
}
