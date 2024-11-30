use crate::analysis::inference_status::InferenceStatusReport;
use crate::analysis::inference_type::InferenceType;
use crate::analysis::update_fn_details::MAX_UPDATE_FN_COUNT;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

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

    /// Prepare a formated summary of inference results, basically a "report" on the
    /// computation progress and results.
    pub fn format_to_report(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Number of satisfying candidates: {}\n",
            self.num_sat_networks
        ));
        output.push_str(&format!(
            "Computation time: {} milliseconds\n\n",
            self.comp_time
        ));
        output.push_str("--------------\n");
        output.push_str("Extended summary:\n");
        output.push_str("--------------\n");
        output.push_str(&format!("{}\n", self.summary_message));

        output.push_str("--------------\n");
        output.push_str("Number of admissible update functions per variable:\n");
        output.push_str("--------------\n");
        let mut sorted_vars: Vec<_> = self.num_update_fns_per_var.iter().collect();
        sorted_vars.sort_by_key(|&(var, _)| var);
        for (var, &count) in sorted_vars {
            let count_display = if count >= MAX_UPDATE_FN_COUNT {
                format!("more than {MAX_UPDATE_FN_COUNT}")
            } else {
                count.to_string()
            };
            output.push_str(&format!("{}: {}\n", var, count_display));
        }

        output.push_str("--------------\n");
        output.push_str("Detailed progress report:\n");
        output.push_str("--------------\n");
        for report in &self.progress_statuses {
            output.push_str(&report.message);
            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use crate::analysis::inference_results::InferenceResults;
    use crate::analysis::inference_status::{InferenceStatus, InferenceStatusReport};
    use crate::analysis::inference_type::InferenceType;

    #[test]
    fn test_inference_results_summary_and_report() {
        // Create a basic instance of InferenceResults
        let mut inference_results = InferenceResults::new(
            InferenceType::FullInference,
            5,
            Duration::from_millis(1500),
            "Initial summary.",
            vec![
                InferenceStatusReport::new(InferenceStatus::Created, None, 0, "Started"),
                InferenceStatusReport::new(
                    InferenceStatus::FinishedSuccessfully,
                    Some(5),
                    1500,
                    "Finished",
                ),
            ],
            HashMap::from([("var1".to_string(), 3), ("var2".to_string(), 7)]),
        );

        // Test extending the summary
        inference_results.extend_summary(" Additional details.");
        assert_eq!(
            inference_results.summary_message,
            "Initial summary. Additional details."
        );

        // Test formatting the report
        let report = inference_results.format_to_report();
        assert!(report.contains("Number of satisfying candidates: 5"));
        assert!(report.contains("Computation time: 1500 milliseconds"));
        assert!(report.contains("Initial summary. Additional details."));
        assert!(report.contains("var1: 3"));
        assert!(report.contains("var2: 7"));
        assert!(report.contains("Started"));
        assert!(report.contains("Finished"));
    }
}
