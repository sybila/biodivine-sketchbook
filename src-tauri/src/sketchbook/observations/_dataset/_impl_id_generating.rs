use crate::sketchbook::ids::{ObservationId, VarId};
use crate::sketchbook::observations::Dataset;
use crate::sketchbook::Manager;

/// Methods for safely generating new valid (unique) instances of identifiers for
/// the current `Dataset`.
impl Dataset {
    /// Generate valid `ObservationId` that's currently not used by any observation in this
    /// `Dataset`.
    ///
    /// First, the given `ideal_id` and its transformation by replacing invalid characters are
    /// tried. If they are both invalid (non-unique), a numerical identifier is added at the end.
    /// By specifying `start_index`, the index search starts directly at that number (e.g., when
    /// ideal ID is "obs" and start index is 3, search for ID starts with "obs_3", "obs_4", ...)
    ///
    /// **Warning:** Do not use this to pre-generate more than one id at a time, as the process
    /// is deterministic and might generate the same IDs. Always generate an Id, add that
    /// observation, and then repeat for other observations.
    pub fn generate_obs_id(&self, ideal_id: &str, start_index: Option<usize>) -> ObservationId {
        self.generate_id(
            ideal_id,
            &(Self::is_valid_observation),
            self.num_observations(),
            start_index,
        )
    }

    /// Generate valid `VarId` that's currently not used by any variable in this `Dataset`.
    ///
    /// First, the given `ideal_id` and its transformation by replacing invalid characters are
    /// tried. If they are both invalid (non-unique), a numerical identifier is added at the end.
    /// By specifying `start_index`, the index search starts directly at that number (e.g., when
    /// ideal ID is "var" and start index is 3, search for ID starts with "var_3", "var_4", ...)
    ///
    /// **Warning:** Do not use this to pre-generate more than one id at a time, as the process
    /// is deterministic and might generate the same IDs. Always generate an Id, add that
    /// variable, and then repeat for other variables.
    pub fn generate_var_id(&self, ideal_id: &str, start_index: Option<usize>) -> VarId {
        self.generate_id(
            ideal_id,
            &(Self::is_valid_variable),
            self.num_variables(),
            start_index,
        )
    }
}
