use crate::sketchbook::ids::{DatasetId, ObservationId, VarId};
use crate::sketchbook::observations::ObservationManager;
use crate::sketchbook::Manager;

/// Methods for safely generating new valid (unique) instances of identifiers for
/// the current `ObservationManager`.
impl ObservationManager {
    /// Generate valid `DatasetId` that's currently not used by any dataset in this
    /// `ObservationManager`.
    ///
    /// First, the given `ideal_id` or its transformation by replacing invalid characters are tried.
    /// If they are both invalid (non-unique), a numerical identifier is added at the end.
    /// By specifying `start_index`, the index search starts directly at that number (e.g., when
    /// ideal ID is "d" and start index is 3, search for ID starts with "d_3", "d_4", ...)
    ///
    /// **Warning:** Do not use this to pre-generate more than one id at a time, as the process
    /// is deterministic and might generate the same IDs. Always generate an Id, add that dataset,
    /// and then repeat for other datasets.
    pub fn generate_dataset_id(&self, ideal_id: &str, start_index: Option<usize>) -> DatasetId {
        self.generate_id(
            ideal_id,
            &(Self::is_valid_dataset_id),
            self.num_datasets(),
            start_index,
        )
    }

    /// Generate valid `ObservationId` that's currently not used by any observation in a
    /// particular dataset of this `ObservationManager`.
    ///
    /// For more, see [super::Dataset::generate_obs_id].
    ///
    /// Assumes that `dataset_id` was checked beforehand.
    pub fn generate_obs_id(
        &self,
        dataset_id: &DatasetId,
        ideal_obs_id: &str,
        start_index: Option<usize>,
    ) -> ObservationId {
        self.datasets
            .get(dataset_id)
            .unwrap()
            .generate_obs_id(ideal_obs_id, start_index)
    }

    /// Generate valid `VarId` that's currently not used by any variable in a particular
    /// dataset of this `ObservationManager`.
    ///
    /// Assumes that `dataset_id` was checked beforehand.
    pub fn generate_var_id(
        &self,
        dataset_id: &DatasetId,
        ideal_var_id: &str,
        start_index: Option<usize>,
    ) -> VarId {
        self.datasets
            .get(dataset_id)
            .unwrap()
            .generate_var_id(ideal_var_id, start_index)
    }
}
