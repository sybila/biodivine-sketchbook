use crate::app::event::Event;
use crate::app::DynError;

/// A [Consumed] object describes possible outcomes of trying to consume an [Event]
/// by some session state object.
#[derive(Debug)]
pub enum Consumed {
    /// Event was successfully consumed, resulting the provided `state_change` [Event].
    ///
    /// Furthermore, the operation is reversible. This means it can be save to the back-stack
    /// as a pair of `(perform, reverse)` events. Note that the `perform` event does not
    /// *need* to be the original event. For example, there could be automated normalization
    /// which won't need to be repeated.
    ///
    /// At the moment, each event must be reversible by a single event. If this is not the
    /// case, you can "restart" the evaluation process with a new, more granular event chain
    /// by returning [Consumed::Restart]. It is the responsibility of the session state to
    /// record this whole chain as a single reversible action.
    ///
    /// However, note that this does not *guarantee* that the action will be saved to the
    /// undo stack. If the payloads for the `(perform, reverse)` actions are too large,
    /// session state can still refuse to save it, in which case the stack will be simply
    /// reset to empty. Finally, some sessions may not have an undo stack at all, in which case
    /// the `(perform, reverse)` pair is completely ignored.
    Reversible {
        state_change: Event,
        perform_reverse: (Event, Event),
    },

    /// Action was successfully consumed, resulting in the given `state_change` [Event].
    ///
    /// However, the action is irreversible. This means the undo stack should be either
    /// cleared if `reset=true`, or the action should bypass the stack completely
    /// if `reset=false`.
    Irreversible { state_change: Event, reset: bool },

    /// The action was successfully consumed (same as [Consumed::Irreversible], but there
    /// is an additional warning sent to the frontend.
    IrreversibleWithWarning {
        state_change: Event,
        reset: bool,
        warning: String,
    },

    /// Action cannot be consumed as is and should be instead replaced by the provided
    /// list of events.
    ///
    /// Note that the original event may or may not be a member of this list. In particular,
    /// if you want to retry the event, you have to manually copy it into the list.
    ///
    /// You can use this result type to perform additional events that are necessary to execute
    /// before the original event can be completed safely. For example, if we want to delete
    /// a variable, we want to first delete all associated regulations.
    Restart(Vec<Event>),

    /// The action was consumed, but the provided user input is invalid and cannot be applied.
    ///
    /// Note that this should only be used when the *user input* is wrong. If some other part
    /// of the action is wrong (e.g. unknown path, missing fields in payload), the action cannot
    /// be consumed!
    InputError(DynError),

    /// The action was consumed, but the application state did not change.
    NoChange,
}
