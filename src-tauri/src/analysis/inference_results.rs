use crate::analysis::inference_type::InferenceType;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

use super::inference_status::InferenceStatusReport;

/// Object encompassing inference results that are to be send to frontend.
/// It does not contain any intermediate or raw results, these are kept on backend only.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InferenceResults {
    /// Type of the conducted inference analysis.
    pub analysis_type: InferenceType,
    /// Number of satisfying networks.
    pub num_sat_networks: u128,
    /// Computation time in miliseconds.
    pub comp_time: u128,
    /// String message summarizing computation to be displayed/logged on frontend.
    pub summary_message: String,
    /// All status updates of the solver (from creation to finish).
    pub progress_statuses: Vec<InferenceStatusReport>,
    /// Number of admissible update functions per each variable.
    pub num_update_fns_per_var: HashMap<String, usize>,
}

impl<'de> JsonSerde<'de> for InferenceResults {}

impl InferenceResults {
    /// Create new `InferenceResults` given all the details.
    pub fn new(
        analysis_type: InferenceType,
        num_sat_networks: u128,
        comp_time: Duration,
        summary_message: &str,
        progress_statuses: Vec<InferenceStatusReport>,
        num_update_fns_per_var: HashMap<String, usize>,
    ) -> InferenceResults {
        InferenceResults {
            analysis_type,
            num_sat_networks,
            comp_time: comp_time.as_millis(),
            summary_message: summary_message.to_string(),
            progress_statuses,
            num_update_fns_per_var,
        }
    }

    /// Append string to the end of current metadata.
    pub fn extend_summary(&mut self, new_message: &str) {
        self.summary_message.push_str(new_message);
    }
}
