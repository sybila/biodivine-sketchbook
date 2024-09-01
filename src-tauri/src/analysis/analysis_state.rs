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
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tauri::async_runtime::RwLock;

use super::inference_solver::FinishedInferenceSolver;

/// Object encompassing all of the components of the Analysis tab.
/// That inludes boths the components that are exchanged with frontend,
/// and raw low-level structures used during computation like symbolic graph
/// and its colors.
pub struct AnalysisState {
    /// Boolean network sketch to run the analysis on. Can be a placeholder at the beginning.
    sketch: Sketch,
    /// Flag signalling that the actual sketch data were received from editor session.
    sketch_received: bool,
    /// Potential analysis solver instance.
    solver: Option<Arc<RwLock<InferenceSolver>>>,
    /// Potential channel to receive (text) updates from the solver instance.
    receiver_channel: Option<Receiver<String>>,
    /// Copy of already finished analysis solver instance, used to work with full analysis results.
    /// If the inference ends with error, the error message is stored instead.
    finished_solver: Option<Result<FinishedInferenceSolver, String>>,
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
            receiver_channel: None,
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
            receiver_channel: None,
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
        if let Some(Ok(solver)) = &self.finished_solver {
            let results = solver.results.clone();
            Ok(results)
        } else if let Some(Err(e)) = &self.finished_solver {
            Err(e.clone())
        } else {
            Err("Trying to get results that are not yet computed/fetched.".to_string())
        }
    }
}

/// More complex methods involving dealing with async solver.
impl AnalysisState {
    /// If a computation solver is running, send cancellation flag to it. This is done
    /// asynchronously and might not happen immediately.
    ///
    /// At the same time, all the inference-related fields of this `AnalysisState` are reset.
    /// That is solver and results. The sketch stays the same.
    pub fn initiate_reset(&mut self) {
        if let Some(solver) = &self.solver {
            let solver: Arc<RwLock<InferenceSolver>> = Arc::clone(solver);
            tokio::spawn(async move {
                // todo: currently, we wait to achieve this "write lock" until whole inference finishes...
                solver.write().await.cancel();
            });
        }
        self.solver = None;
        self.receiver_channel = None;
        self.finished_solver = None;
        self.results = None;
    }

    /// Check if the inference solver finished its computation. If so, clone the important parts
    /// of the solver into `finished_solver` field (so we can easily access it). Return error if
    /// the fetch is unsuccessful because the computation is still running.
    ///
    /// Once the OK(()) is returned by this method, we know the computation is over, the results
    /// are fetched into `finished_solver` attribute and we can safely access them. If some
    /// error happened during inference computation, the `finished_solver` field contains an Err.
    pub fn try_fetch_results(&mut self) -> Result<(), String> {
        if self.finished_solver.is_some() {
            debug!("Full results were already fetched. Not trying to fetch them again.");
            return Ok(());
        }

        if let Some(solver) = self.solver.clone() {
            // computation is finished when we can obtain lock and `solver.is_finished()` is true
            if let Ok(solver) = solver.try_read() {
                if !solver.is_finished() {
                    return Err("Computation is still running.".to_string());
                }

                self.finished_solver = Some(solver.to_finished_solver());
                debug!(
                    "Successfully fetched results from solver (they still might contain error)."
                );
                Ok(())
            } else {
                Err("Computation is still running.".to_string())
            }
        } else {
            Err("No computation is running.".to_string())
        }
    }

    /// Check if there are any new messages from the solver (reporting on its progress).
    /// There can be more than one message. Each message is appended with a newline, and if there
    /// is more than one, they are combined.
    ///
    /// Return error if there is no new message, computation is finished, or it was not started yet.
    pub fn try_get_solver_progress(&mut self) -> Result<String, String> {
        if self.finished_solver.is_some() {
            return Err(
                "Full results were already fetched. Not trying to fetch progress.".to_string(),
            );
        }

        if let Some(receiver_channel) = self.receiver_channel.as_mut() {
            let mut message = String::new();
            while let Ok(msg) = receiver_channel.try_recv() {
                message.push_str(&msg);
                message.push('\n');
            }

            if message.is_empty() {
                Err("No new message was sent.".to_string())
            } else {
                Ok(message)
            }
        } else {
            Err("No computation is running.".to_string())
        }
    }

    /// Start the inference computation on a separate thread. If some previous computation is
    /// running, it is cancelled first.
    ///
    /// The computation solver has its own thread. Method [try_fetch_results] can be used to
    /// test if the results are ready (and fetch them if so). Method [try_get_solver_progress]
    /// can be used to collect progress messages sent from the solver.
    pub fn start_analysis(&mut self, analysis_type: AnalysisType) -> Result<(), DynError> {
        if !self.sketch_received || self.sketch.model.num_vars() == 0 {
            return AeonError::throw("Cannot run analysis on empty sketch.");
        }

        self.initiate_reset(); // Reset the state before starting new analysis

        let (progress_sender, progress_receiver): (Sender<String>, Receiver<String>) =
            mpsc::channel();
        self.receiver_channel = Some(progress_receiver);
        let solver = Arc::new(RwLock::new(InferenceSolver::new(progress_sender)));
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
                    "Async analysis thread finished. There are {} sat networks.",
                    result.num_sat_networks
                ),
                Err(e) => debug!("Async analysis thread finished with an error: {e}"),
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

                // there are four main scenarios:
                // 1) the solver successfully finished and we try to extract full results and send them to FE
                // 2) the solver finished with error and we send error to FE
                // 3) solver is running, but it at least reported some progress and we send it to FE
                // 4) we just dont send anything cause nothing new happened
                if fetch_res.is_ok() {
                    let state_change = match self.get_results() {
                        Ok(results) => {
                            let payload = results.to_json_str();
                            match results.analysis_type {
                                AnalysisType::Inference => {
                                    Event::build(&["analysis", "inference_results"], Some(&payload))
                                }
                                AnalysisType::StaticCheck => {
                                    Event::build(&["analysis", "static_results"], Some(&payload))
                                }
                                AnalysisType::DynamicCheck => {
                                    Event::build(&["analysis", "dynamic_results"], Some(&payload))
                                }
                            }
                        }
                        Err(message) => {
                            let payload = serde_json::to_string(&message).unwrap();
                            Event::build(&["analysis", "inference_error"], Some(&payload))
                        }
                    };
                    Ok(Consumed::Irreversible {
                        state_change,
                        reset: true,
                    })
                } else if let Ok(message) = self.try_get_solver_progress() {
                    let payload = serde_json::to_string(&message).unwrap();
                    let state_change =
                        Event::build(&["analysis", "computation_update"], Some(&payload));
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

                // this sends cancellation flag to the potentially running inference solver, and resets
                // attributes of this AnalysisState
                self.initiate_reset();

                let state_change = Event::build(&["analysis", "analysis_reset"], Some("true"));
                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            Some(&"sample_networks") => {
                let payload = Self::clone_payload_str(event, component)?;
                let sampling_data = SamplingData::from_json_str(&payload)?;

                if let Some(Ok(solver)) = &self.finished_solver {
                    download_witnesses(
                        &sampling_data.path,
                        solver.sat_colors.clone(),
                        &solver.graph,
                        sampling_data.count,
                        sampling_data.seed,
                    )?;
                    Ok(Consumed::NoChange {})
                } else {
                    AeonError::throw(
                        "Cannot sample networks because inference results were not fetched yet (or were erronous).",
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
