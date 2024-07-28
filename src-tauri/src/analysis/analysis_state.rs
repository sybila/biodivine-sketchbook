use crate::analysis::analysis_results::AnalysisResults;
use crate::analysis::inference_solver::InferenceSolver;
use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::debug;
use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::{JsonSerde, Sketch};
use serde::{Deserialize, Serialize};

/// Object encompassing all of the state components of the Analysis itself that are exchanged
/// with frontend (no raw low-level structures like symbolic graph or its colors)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnalysisState {
    /// Boolean network sketch to run the analysis on. Can be a placeholder at the beginning.
    sketch: Sketch,
    /// Flag signalling that the actual sketch data were received from editor session.
    sketch_received: bool,
    /// Potential results of the analysis.
    results: Option<AnalysisResults>,
    // TODO
}

impl<'de> JsonSerde<'de> for AnalysisState {}

impl AnalysisState {
    /// Create new `AnalysisState` with an empty placeholder sketch.
    ///
    /// This is used to create a placeholder instance before the actual sketch data are sent from
    /// the editor session.
    pub fn new_empty() -> AnalysisState {
        AnalysisState {
            sketch: Sketch::default(),
            sketch_received: false,
            results: None,
        }
    }

    /// Create new `AnalysisState` with a full sketch data.
    pub fn new(sketch: Sketch) -> AnalysisState {
        AnalysisState {
            sketch,
            sketch_received: true,
            results: None,
        }
    }

    /// Update the sketch data of this `AnalysisState`.
    pub fn set_sketch(&mut self, sketch: Sketch) {
        self.sketch = sketch;
        self.sketch_received = true;
    }

    /// Get reference to the sketch data of this `AnalysisState`.
    pub fn get_sketch(&self) -> &Sketch {
        &self.sketch
    }
}

impl SessionHelper for AnalysisState {}

impl SessionState for AnalysisState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        let component = "analysis";

        match at_path.first() {
            Some(&"run_inference") => {
                Self::assert_payload_empty(event, component)?;
                debug!("Event `run_inference` received. It is not fully implemented yet.");
                // TODO

                let mut inference_solver = InferenceSolver::new();
                let results = inference_solver.run_computation_prototype(self.sketch.clone())?;

                let payload = results.to_json_str();
                let state_change = Event::build(&["analysis", "inference_results"], Some(&payload));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"run_static") => {
                Self::assert_payload_empty(event, component)?;
                debug!("Event `run_static` received. It is not implemented at the moment. Only dummy message will be sent");
                // TODO

                let state_change = Event::build(&["analysis", "static_running"], Some("true"));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            _ => Self::invalid_path_error_generic(at_path),
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        let component_name = "analysis";

        // currently three options: get all datasets, a single dataset, a single observation
        match at_path.first() {
            Some(&"get_sketch") => {
                Self::assert_path_length(at_path, 1, component_name)?;
                let sketch_data = SketchData::new_from_sketch(&self.sketch);
                Ok(Event {
                    path: full_path.to_vec(),
                    payload: Some(sketch_data.to_json_str()),
                })
            }
            _ => Self::invalid_path_error_generic(at_path),
        }
    }
}
