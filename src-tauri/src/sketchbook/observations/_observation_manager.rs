use crate::sketchbook::observations::ObservationList;
use crate::sketchbook::ObservationListId;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct ObservationManager {
    datasets: HashMap<ObservationListId, ObservationList>,
}

impl Default for ObservationManager {
    /// Default object with no datasets.
    fn default() -> ObservationManager {
        ObservationManager::new_empty()
    }
}

impl ObservationManager {
    /// Instantiate `ObservationManager` with empty list of datasets.
    pub fn new_empty() -> ObservationManager {
        ObservationManager {
            datasets: HashMap::new(),
        }
    }
}
