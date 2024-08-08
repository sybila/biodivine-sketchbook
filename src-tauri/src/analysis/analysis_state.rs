use crate::analysis::analysis_results::AnalysisResults;
use crate::analysis::data_structs::SamplingData;
use crate::analysis::inference_solver::InferenceSolver;
use crate::analysis::sampling_networks::download_witnesses;
use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::{AeonError, DynError};
use crate::debug;
use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::{JsonSerde, Sketch};

/// Object encompassing all of the components of the Analysis itself
/// That inludes boths the components that are exchanged with frontend
/// and also raw low-level structures like symbolic graph or its colors.
#[derive(Clone)]
pub struct AnalysisState {
    /// Boolean network sketch to run the analysis on. Can be a placeholder at the beginning.
    sketch: Sketch,
    /// Flag signalling that the actual sketch data were received from editor session.
    sketch_received: bool,
    /// Potential analysis solver instance.
    solver: Option<InferenceSolver>,
    /// Potential results of the analysis.
    results: Option<AnalysisResults>,
    // TODO
}

impl AnalysisState {
    /// Create new `AnalysisState` with an empty placeholder sketch.
    ///
    /// This is used to create a placeholder instance before the actual sketch data are sent from
    /// the editor session.
    pub fn new_empty() -> AnalysisState {
        AnalysisState {
            sketch: Sketch::default(),
            sketch_received: false,
            solver: None,
            results: None,
        }
    }

    /// Create new `AnalysisState` with a full sketch data.
    pub fn new(sketch: Sketch) -> AnalysisState {
        AnalysisState {
            sketch,
            sketch_received: true,
            solver: None,
            results: None,
        }
    }

    /// Reset the results and analyses of this `AnalysisState`.
    /// The sketch data stays the same.
    pub fn reset(&mut self) {
        self.results = None;
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

                if !self.sketch_received || self.sketch.model.num_vars() == 0 {
                    return AeonError::throw("Cannot run analysis on empty sketch.");
                }

                self.solver = Some(InferenceSolver::new());
                let results = self
                    .solver
                    .as_mut()
                    .unwrap()
                    .run_whole_inference_prototype(self.sketch.clone())?;

                let payload = results.to_json_str();
                let state_change = Event::build(&["analysis", "inference_results"], Some(&payload));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"run_static") => {
                Self::assert_payload_empty(event, component)?;
                debug!("Event `run_static` received. It is not implemented at the moment. Only dummy message will be sent back.");

                // TODO

                let state_change = Event::build(&["analysis", "static_running"], Some("true"));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"reset_analysis") => {
                Self::assert_payload_empty(event, component)?;

                self.reset();

                let state_change = Event::build(&["analysis", "analysis_reset"], Some("true"));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"sample_networks") => {
                let payload = Self::clone_payload_str(event, component)?;
                let sampling_data = SamplingData::from_json_str(&payload)?;

                if let Some(solver) = &self.solver {
                    download_witnesses(
                        &sampling_data.path,
                        solver.sat_colors()?.clone(),
                        solver.graph()?,
                        sampling_data.count,
                        sampling_data.seed,
                    )?;
                    Ok(Consumed::NoChange {})
                } else {
                    AeonError::throw(
                        "Cannot sample networks because there are no inference results.",
                    )
                }
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
