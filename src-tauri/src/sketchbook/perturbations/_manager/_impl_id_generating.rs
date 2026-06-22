use crate::sketchbook::ids::PerturbationId;
use crate::sketchbook::perturbations::PerturbationManager;
use crate::sketchbook::Manager;

/// Methods for safely generating new valid (unique) instances of identifiers for
/// the current `PerturbationManager`.
impl PerturbationManager {
    /// Generate valid `PerturbationId` that's currently not used by any perturbation in this
    /// `PerturbationManager`.
    ///
    /// First, the given `ideal_id` or its transformation by replacing invalid characters are tried.
    /// If they are both invalid (non-unique), a numerical identifier is added at the end.
    /// By specifying `start_index`, the index search starts directly at that number (e.g., when
    /// ideal ID is "per" and start index is 3, search for ID starts with "per_3", "per_4", ...)
    ///
    /// **Warning:** Do not use this to pre-generate more than one id at a time, as the process
    /// is deterministic and might generate the same IDs. Always generate an Id, add that perturbation,
    /// and then repeat for other perturbations.
    pub fn generate_perturbation_id(
        &self,
        ideal_id: &str,
        start_index: Option<usize>,
    ) -> PerturbationId {
        self.generate_id(
            ideal_id,
            &(Self::is_valid_perturbation_id),
            self.num_perturbations(),
            start_index,
        )
    }
}
