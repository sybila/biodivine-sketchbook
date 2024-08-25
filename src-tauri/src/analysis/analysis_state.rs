use crate::analysis::analysis_results::AnalysisResults;
use crate::analysis::analysis_type::AnalysisType;
use crate::analysis::data_structs::SamplingData;
use crate::analysis::inference_solver::InferenceSolver;
use crate::analysis::sampling_networks::download_witnesses;
use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::{AeonError, DynError};
use crate::debug;
use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::{JsonSerde, Sketch};
use std::sync::Arc;
use tauri::async_runtime::RwLock;

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
    solver: Option<Arc<RwLock<InferenceSolver>>>,
    /// Copy of already finished analysis solver instance, used to work with full analysis results.
    finished_solver: Option<InferenceSolver>,
    /// Potential simplified processed results of the analysis.
    results: Option<AnalysisResults>,
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
            finished_solver: None,
            results: None,
        }
    }

    /// Create new `AnalysisState` with a full sketch data.
    pub fn new(sketch: Sketch) -> AnalysisState {
        AnalysisState {
            sketch,
            sketch_received: true,
            solver: None,
            finished_solver: None,
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

    /// Getter for pre-processed results from the internal solver.
    /// If the results were not fetched yet (analysis still running), returns error.
    ///
    /// This method is only a simple getter. See [try_fetch_results] for actual result fetching.
    pub fn get_results(&self) -> Result<AnalysisResults, String> {
        if let Some(solver) = &self.finished_solver {
            let results = solver.results()?;
            Ok(results.clone())
        } else {
            Err("Results not yet computed/fetched.".to_string())
        }
    }
}

/// More complex methods involving dealing with async solver.
impl AnalysisState {
    /// If a computation is running, set cancellation flag to it. This is done asynchronously
    /// and might not happen immediately.
    ///
    /// At the same time, all the inference-related fields of this `AnalysisState` are reset.
    /// That is solver and results. The sketch stays the same.
    pub fn start_reset(&mut self) {
        if let Some(solver) = &self.solver {
            let solver: Arc<RwLock<InferenceSolver>> = Arc::clone(solver);
            tokio::spawn(async move {
                // todo: currently, we wait to achieve this "write lock" until whole inference finishes...
                solver.write().await.cancel();
            });
        }
        self.solver = None;
        self.finished_solver = None;
        self.results = None;
    }

    pub fn try_fetch_results(&mut self) -> Result<(), String> {
        if self.finished_solver.is_some() {
            debug!("Results were already fetched. Not trying again.");
            return Ok(());
        }

        if let Some(solver) = self.solver.clone() {
            if let Ok(solver) = solver.try_read() {
                self.finished_solver = Some(solver.clone());
                debug!("Successfully fetched results.");
                Ok(())
            } else {
                Err("Computation is still running.".to_string())
            }
        } else {
            Err("No computation is running.".to_string())
        }
    }

    pub fn start_analysis(&mut self, analysis_type: AnalysisType) -> Result<(), DynError> {
        if !self.sketch_received || self.sketch.model.num_vars() == 0 {
            return AeonError::throw("Cannot run analysis on empty sketch.");
        }

        self.start_reset(); // Reset the state before starting new analysis
        let solver = Arc::new(RwLock::new(InferenceSolver::new()));
        self.solver = Some(Arc::clone(&solver));
        let sketch = self.sketch.clone();

        // Capture only the necessary data for the async block
        let solver_clone = Arc::clone(&solver);

        // Spawn the async task for the analysis
        tokio::spawn(async move {
            // this saves the results internally to the solver instance
            let results =
                InferenceSolver::run_inference_async(solver_clone, sketch, analysis_type).await;

            // this is just for debugging purposes
            match results {
                Ok(result) => debug!(
                    "Async analysis computation finished. There are {} sat networks.",
                    result.num_sat_networks
                ),
                Err(e) => debug!("Async analysis computation finished with an error: {e}"),
            }
        });
        Ok(())
    }
}

impl SessionHelper for AnalysisState {}

impl SessionState for AnalysisState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        let component = "analysis";

        match at_path.first() {
            Some(&"run_full_inference") => {
                Self::assert_payload_empty(event, "analysis")?;
                debug!(
                    "Event `run_inference` received. Starting full inference with all properties."
                );

                self.start_analysis(AnalysisType::Inference)?; // Start analysis and handle asynchronously
                let state_change = Event::build(&["analysis", "inference_running"], Some("true"));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"run_partial_static") => {
                Self::assert_payload_empty(event, "analysis")?;
                debug!("Event `run_partial_static` received. Starting partial inference with static properties.");

                self.start_analysis(AnalysisType::StaticCheck)?; // Start analysis and handle asynchronously
                let state_change = Event::build(&["analysis", "static_running"], Some("true"));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"run_partial_dynamic") => {
                Self::assert_payload_empty(event, "analysis")?;
                debug!("Event `run_partial_dynamic` received. Starting partial inference with dynamic properties.");

                self.start_analysis(AnalysisType::DynamicCheck)?; // Start analysis and handle asynchronously
                let state_change = Event::build(&["analysis", "dynamic_running"], Some("true"));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"get_inference_results") => {
                // Note that this event can be used to retrieve results of any running analysis, be it
                // full inference, static check, or dynamic check.
                // The type of state change event is decided based on what kind of analysis was running.

                Self::assert_payload_empty(event, component)?;
                //debug!("Event `get_inference_results` received. Trying to fetch inference results.");

                let fetch_res = self.try_fetch_results();

                // either the results are ready and we send them back, or we just dont send anything
                if fetch_res.is_ok() {
                    let results = self.get_results()?;
                    let payload = results.to_json_str();
                    let state_change = match results.analysis_type {
                        AnalysisType::Inference => {
                            Event::build(&["analysis", "inference_results"], Some(&payload))
                        }
                        AnalysisType::StaticCheck => {
                            Event::build(&["analysis", "static_results"], Some(&payload))
                        }
                        AnalysisType::DynamicCheck => {
                            Event::build(&["analysis", "dynamic_results"], Some(&payload))
                        }
                    };
                    Ok(Consumed::Irreversible {
                        state_change,
                        reset: true,
                    })
                } else {
                    Ok(Consumed::NoChange)
                }
            }
            Some(&"reset_analysis") => {
                Self::assert_payload_empty(event, component)?;

                self.start_reset();

                let state_change = Event::build(&["analysis", "analysis_reset"], Some("true"));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"sample_networks") => {
                let payload = Self::clone_payload_str(event, component)?;
                let sampling_data = SamplingData::from_json_str(&payload)?;

                if let Some(solver) = &self.finished_solver {
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
                        "Cannot sample networks because inference results were not fetched yets.",
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
