use crate::algorithms::eval_dynamic::eval::eval_dyn_prop;
use crate::algorithms::eval_dynamic::prepare_graph::prepare_graph_for_dynamic_hctl;
use crate::algorithms::eval_dynamic::processed_props::{process_dynamic_props, ProcessedDynProp};
use crate::algorithms::eval_static::eval::eval_static_prop;
use crate::algorithms::eval_static::prepare_graph::prepare_graph_for_static_fol;
use crate::algorithms::eval_static::processed_props::{process_static_props, ProcessedStatProp};
use crate::algorithms::fo_logic::utils::get_implicit_function_name;
use crate::debug;
use crate::inference::inference_results::InferenceResults;
use crate::inference::inference_status::InferenceStatus;
use crate::inference::inference_type::InferenceType;
use crate::sketchbook::{JsonSerde, Sketch};
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::BooleanNetwork;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tauri::async_runtime::RwLock;

use super::inference_status::InferenceStatusReport;
use super::update_fn_details::num_update_fn_variants_per_var;

/// Object encompassing the process of the BN inference computation.
///
/// It tracks the intermediate results and low-level structures, and it provides hooks to the
/// actual algorithms.
///
/// By tracking the intermediate results, we may be able to observe the computation, and potentially
/// "fork" or re-use parts of the computation in future. The computation is made to be run
/// asynchronously itself on a special thread, and the solver reports on progress via a special
/// channel.
pub struct InferenceSolver {
    /// Boolean Network instance (once processed).
    bn: Option<BooleanNetwork>,
    /// Symbolic transition graph for the system.
    /// Its set of unit colors is gradually updated during computation, and it consists of
    /// currently remaining valid candidate colors.
    graph: Option<SymbolicAsyncGraph>,
    /// Start time of the computation (once started).
    start_time: Option<SystemTime>,
    /// Static properties (once processed).
    static_props: Option<Vec<ProcessedStatProp>>,
    /// Dynamic properties (once processed).
    dynamic_props: Option<Vec<ProcessedDynProp>>,
    /// Set of final satisfying colors ((if computation finishes successfully).
    raw_sat_colors: Option<GraphColors>,
    /// Vector with all time-stamped status updates. The last is the latest status.
    status_updates: Vec<InferenceStatusReport>,
    /// Flag to signal cancellation.
    should_stop: Arc<AtomicBool>,
    /// Channel to send updates regarding the computation.
    sender_channel: Sender<String>,
    /// Potential processed results (if computation finishes successfully).
    results: Option<InferenceResults>,
    /// Potential error message (if computation finishes with error).
    error_message: Option<String>,
}

/// Object encompassing a finished (successful) BN inference computation with all
/// intermediate and processed results.
///
/// It is essentially a simplified version of `InferenceSolver` that can be used
/// to easily observe results and for sampling.
#[derive(Clone)]
pub struct FinishedInferenceSolver {
    pub bn: BooleanNetwork,
    pub graph: SymbolicAsyncGraph,
    pub sat_colors: GraphColors,
    pub results: InferenceResults,
}

/// Basic utilities, constructors, getters, and so on.
impl InferenceSolver {
    /// Prepares new "empty" `InferenceSolver` instance that can be later used to
    /// run the computation.
    ///
    /// Currently, the computation can be started by the `run_inference_async` method.
    pub fn new(sender_channel: Sender<String>) -> InferenceSolver {
        let msg = "Created solver instance";
        let initial_status = InferenceStatusReport::new(InferenceStatus::Created, None, 0, msg);
        InferenceSolver {
            bn: None,
            graph: None,
            start_time: None,
            static_props: None,
            dynamic_props: None,
            raw_sat_colors: None,
            status_updates: vec![initial_status],
            should_stop: Arc::new(AtomicBool::new(false)),
            sender_channel,
            results: None,
            error_message: None,
        }
    }

    /// Reference getter for a Boolean network.
    pub fn bn(&self) -> Result<&BooleanNetwork, String> {
        if let Some(bn) = &self.bn {
            Ok(bn)
        } else {
            Err("Boolean network not yet processed.".to_string())
        }
    }

    /// Reference getter for a transition graph.
    pub fn graph(&self) -> Result<&SymbolicAsyncGraph, String> {
        if let Some(graph) = &self.graph {
            Ok(graph)
        } else {
            Err("Transition graph and symbolic context not yet computed.".to_string())
        }
    }

    /// Reference getter for a vector of formulas for static properties.
    pub fn stat_props(&self) -> Result<&Vec<ProcessedStatProp>, String> {
        if let Some(stat_props) = &self.static_props {
            Ok(stat_props)
        } else {
            Err("Static properties not yet processed.".to_string())
        }
    }

    /// Reference getter for a vector of formulas for dynamic properties.
    pub fn dyn_props(&self) -> Result<&Vec<ProcessedDynProp>, String> {
        if let Some(dyn_props) = &self.dynamic_props {
            Ok(dyn_props)
        } else {
            Err("Dynamic properties not yet processed.".to_string())
        }
    }

    /// Reference getter for a set of satisfying graph colors.
    pub fn final_sat_colors(&self) -> Result<&GraphColors, String> {
        if let Some(colors) = &self.raw_sat_colors {
            Ok(colors)
        } else {
            Err("Satisfying colors not yet computed.".to_string())
        }
    }

    /// Get a current set of valid candidate colors. This can be used to get intermediate
    /// results and is gradually updated during computation.
    pub fn current_candidate_colors(&self) -> Result<GraphColors, String> {
        Ok(self.graph()?.mk_unit_colors())
    }

    /// Get a start time of the actual computation.
    pub fn start_time(&self) -> Result<SystemTime, String> {
        self.start_time
            .ok_or("Computation not yet started.".to_string())
    }

    /// Get a total duration of the actual inference computation.
    pub fn total_duration(&self) -> Result<Duration, String> {
        // there is always some status (since one is given during initialization)
        let last_status = self.status_updates.last().unwrap();
        match last_status.status {
            InferenceStatus::FinishedSuccessfully => {
                let num_millis = last_status.comp_time;
                Ok(Duration::from_millis(num_millis as u64))
            }
            InferenceStatus::Error => {
                Err("Computation failed to finish because there was an error.".to_string())
            }
            _ => Err("Computation not yet finished.".to_string()),
        }
    }

    /// Update the status of the solver, and send a progress message to the InferenceState
    /// instance (that started this solver).
    ///
    /// If the channel for progress updates no longer exists (because inference is supposed to
    /// be reset, the window was closed, or some other reason), we instead forcibly stop the
    /// computation. Destroying the channel can thus actually be used as another way to stop the
    /// asynchronous computation, since one does not need to acquire lock over the whole solver.
    fn update_status(&mut self, status: InferenceStatus) {
        // starting time must be saved before any statuses are added
        let start_time = self.start_time().unwrap();
        let now = SystemTime::now();
        let duration_since_start = now.duration_since(start_time).unwrap();
        let duration_millis = duration_since_start.as_millis();

        let candidates_num = self
            .current_candidate_colors()
            .ok()
            .map(|x| x.exact_cardinality().to_string());
        let message = self.format_status_message(&status, duration_millis, candidates_num.clone());
        debug!("{message}");

        let status_report =
            InferenceStatusReport::new(status.clone(), candidates_num, duration_millis, &message);
        let status_json = status_report.to_json_str();
        self.status_updates.push(status_report);

        // send JSON string to the channel so it can be send to the frontend later
        // if the channel for progress updates does not exist anymore, stop computation
        if self.sender_channel.send(status_json).is_err() {
            self.cancel();
        }
    }

    /// Format a computation status message.
    fn format_status_message(
        &self,
        status: &InferenceStatus,
        comp_time: u128,
        num_candidates: Option<String>,
    ) -> String {
        let candidates_str = if num_candidates.is_some() & requires_candidate_num(status) {
            let num = num_candidates.unwrap();
            format!(" ({} candidates)", num)
        } else {
            String::new()
        };

        let msg = match &status {
            InferenceStatus::Created => "Created solver instance".to_string(),
            InferenceStatus::Started => "Started inference computation".to_string(),
            InferenceStatus::ProcessedInputs => "Pre-processed all inputs".to_string(),
            InferenceStatus::GeneratedContextStatic => {
                "Starting to evaluate static properties".to_string()
            }
            InferenceStatus::GeneratedContextDynamic => {
                "Starting to evaluate dynamic properties".to_string()
            }
            InferenceStatus::EvaluatedStatic(id) => format!("Evaluated static property `{id}`"),
            InferenceStatus::EvaluatedDynamic(id) => format!("Evaluated dynamic property `{id}`"),
            InferenceStatus::EvaluatedAllStatic => "Evaluated all static properties".to_string(),
            InferenceStatus::EvaluatedAllDynamic => "Evaluated all dynamic properties".to_string(),
            InferenceStatus::DetectedUnsat => "Found that sketch is unsatisfiable".to_string(),
            InferenceStatus::FinishedSuccessfully => {
                "Successfully finished computation".to_string()
            }
            InferenceStatus::Error => "Encountered error during computation".to_string(),
        };
        format!("> {comp_time}ms: {msg}{candidates_str}")
    }

    /// Utility to check whether the cancellation flag was set. If it is set, the function
    /// returns error. Otherwise, nothing happens.
    fn check_cancellation(&self) -> Result<(), String> {
        if self.should_stop.load(Ordering::SeqCst) {
            return Err("Computation was cancelled.".to_string());
        }
        Ok(())
    }

    /// Utility to check whether the sketch (during computation) is already found to be
    /// unsatisfiable.
    ///
    /// If arg `update_status`` is true and sketch is unsat, the status is update with
    ///  [InferenceStatus::DetectedUnsat].
    ///
    /// This method only makes sense when the computation is already on the way.
    fn check_if_finished_unsat(&mut self, update_status: bool) -> Result<bool, String> {
        if let Ok(candidate_set) = self.current_candidate_colors() {
            let unsat = candidate_set.exact_cardinality() == BigInt::from(0);
            if update_status && unsat {
                self.update_status(InferenceStatus::DetectedUnsat);
            }
            Ok(unsat)
        } else {
            Err("Computation did not start yet.".to_string())
        }
    }

    /// If computation successfully finished, transform into `FinishedInferenceSolver`.
    /// Otherwise, return the error with the error message that caused inference fail.
    pub fn to_finished_solver(&self) -> Result<FinishedInferenceSolver, String> {
        // there is always at least 1 status, we can unwrap
        let last_status = self.status_updates.last().unwrap();

        // only convert if the last status is Finished, otherwise throw error
        match last_status.status {
            InferenceStatus::FinishedSuccessfully => Ok(FinishedInferenceSolver {
                bn: self.bn.clone().unwrap(),
                graph: self.graph.clone().unwrap(),
                sat_colors: self.raw_sat_colors.clone().unwrap(),
                results: self.results.clone().unwrap(),
            }),
            InferenceStatus::Error => {
                // check if the real error message was stored, or use default message
                if let Some(msg) = &self.error_message {
                    Err(msg.clone())
                } else {
                    Err("Computation ended up with an internal error.".to_string())
                }
            }
            _ => Err("Computation not yet finished.".to_string()),
        }
    }

    /// Check if computation finished (by success or error).
    pub fn is_finished(&self) -> bool {
        // there is always at least 1 status, we can unwrap
        let last_status = self.status_updates.last().unwrap();
        matches!(
            last_status.status,
            InferenceStatus::FinishedSuccessfully | InferenceStatus::Error
        )
    }

    /// Number of dynamic properties that were already successfully evaluated.
    pub fn num_finished_dyn_props(&self) -> u64 {
        self.status_updates.iter().fold(0, |accum, status| {
            if matches!(status.status, InferenceStatus::EvaluatedDynamic(..)) {
                accum + 1
            } else {
                accum
            }
        })
    }

    /// Number of static properties that were already successfully evaluated.
    pub fn num_finished_stat_props(&self) -> u64 {
        self.status_updates.iter().fold(0, |accum, status| {
            if matches!(status.status, InferenceStatus::EvaluatedStatic(..)) {
                accum + 1
            } else {
                accum
            }
        })
    }
}

/// Methods for asynchronous manipulation of `InferenceSolver` (starting/cancelling inference).
impl InferenceSolver {
    /// Run the prototype version of the inference using the given solver.
    /// This wraps the [Self::run_inference_modular] to also log potential errors.
    ///
    /// The argument `inference_type` specifies which kind of inference should be used.
    /// Currently, we support full inference with all properties, and partial inferences with only
    /// static or only dynamic properties.
    ///
    /// The results are saved to specific fields of the provided solver and can be retrieved later.
    /// They are also returned, which is now used for logging later.
    pub async fn run_inference_async(
        solver: Arc<RwLock<InferenceSolver>>,
        sketch: Sketch,
        inference_type: InferenceType,
    ) -> Result<InferenceResults, String> {
        {
            let solver = solver.read().await;
            solver.check_cancellation()?; // Early check before starting
        }

        // Currently, we use this "write lock" to lock the solver for the whole inference.
        // This works since for sending progress messages we dont need a lock - we use a communication channel.
        // Tthe (non-)existence of the channel as a way to know if the computation was cancelled.

        let mut solver_write = solver.write().await;
        let results = match inference_type {
            InferenceType::FullInference => {
                solver_write.run_inference_modular(inference_type, sketch, true, true)
            }
            InferenceType::StaticInference => {
                solver_write.run_inference_modular(inference_type, sketch, true, false)
            }
            InferenceType::DynamicInference => {
                solver_write.run_inference_modular(inference_type, sketch, false, true)
            }
        };

        // if computation ends with an error, log it
        if let Err(msg) = &results {
            solver_write.error_message = Some(msg.clone());
            solver_write.update_status(InferenceStatus::Error);
        }

        // Lets drop the lock for a bit to allow the potential cancellation thread to achieve a lock.
        // This way we can at least properly check if computation should have been cancelled...
        // (this is currently just to be certain everything is checked, there are now new different
        // methods to cancel the process via channels)
        drop(solver_write);
        thread::sleep(Duration::from_millis(10));

        {
            let solver = solver.read().await;
            solver.check_cancellation()?; // Check if computation should have been cancelled
        }

        results
    }

    /// Set the cancellation flag. The actual cancellation does not happen immediately,
    /// we currently only allow cancelling only at certain checkpoints during computation.
    pub fn cancel(&self) {
        //debug!("`InferenceSolver` has received cancellation flag.");
        self.should_stop.store(true, Ordering::SeqCst);
    }
}

/// Methods related to actual inference computation.
impl InferenceSolver {
    /// Extract and process BN component from the sketch.
    fn extract_bn(sketch: &Sketch) -> Result<BooleanNetwork, String> {
        let bn = sketch.model.to_bn_with_plain_regulations(None);
        // remove all unused function symbols, as these would cause problems later
        let mut bn = bn.prune_unused_parameters();
        // add expressions "f_var_N(regulator_1, ..., regulator_M)" instead of all empty updates
        // this gets us rid of "implicit" update functions, and we can only eval "explicit" parameters
        for var in bn.variables().clone() {
            if bn.get_update_function(var).is_none() {
                let var_name = bn.get_variable_name(var).clone();
                let fn_name = get_implicit_function_name(&var_name);
                let inputs = bn.regulators(var);
                bn.add_parameter(&fn_name, inputs.len() as u32).unwrap();
                let input_str = inputs
                    .iter()
                    .map(|v| bn.get_variable_name(*v).clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                let update_fn_str = format!("{fn_name}({input_str})");
                bn.add_string_update_function(&var_name, &update_fn_str)
                    .unwrap();
            }
        }
        Ok(bn)
    }

    /// Evaluate previously collected static properties, and restrict the unit set of the
    /// graph to the set of valid colors.
    ///
    /// If we discover that sketch is unsat early, skip the rest.
    fn eval_static(&mut self, base_var_name: &str) -> Result<(), String> {
        for stat_property in self.stat_props()?.clone() {
            self.check_cancellation()?; // check if cancellation flag was set during computation

            let prop_id = stat_property.id().to_string();
            let inferred_colors = eval_static_prop(stat_property, self.graph()?, base_var_name)?;
            let colored_vertices = GraphColoredVertices::new(
                inferred_colors.into_bdd(),
                self.graph()?.symbolic_context(),
            );
            let new_graph: SymbolicAsyncGraph = self.graph()?.restrict(&colored_vertices);
            self.graph = Some(new_graph);
            self.update_status(InferenceStatus::EvaluatedStatic(prop_id));
            if self.check_if_finished_unsat(true)? {
                return Ok(());
            }
        }
        self.update_status(InferenceStatus::EvaluatedAllStatic);
        Ok(())
    }

    /// Evaluate previously collected dynamic properties, and restrict the unit set of the
    /// graph to the set of valid colors.
    ///
    /// If we discover that sketch is unsat early, skip the rest.
    fn eval_dynamic(&mut self) -> Result<(), String> {
        for dyn_property in self.dyn_props()?.clone() {
            self.check_cancellation()?; // check if cancellation flag was set during computation

            let prop_id = dyn_property.id().to_string();
            let inferred_colors = eval_dyn_prop(dyn_property, self.graph()?, &track_progress)?;
            let colored_vertices = GraphColoredVertices::new(
                inferred_colors.into_bdd(),
                self.graph()?.symbolic_context(),
            );
            let new_graph: SymbolicAsyncGraph = self.graph()?.restrict(&colored_vertices);
            self.graph = Some(new_graph);
            self.update_status(InferenceStatus::EvaluatedDynamic(prop_id));
            if self.check_if_finished_unsat(true)? {
                return Ok(());
            }
        }
        self.update_status(InferenceStatus::EvaluatedAllDynamic);
        Ok(())
    }

    /// A modular variant of the inference. You can choose which parts to select.
    /// For example, you can only consider static properties, only dynamic properties, or all.
    pub fn run_inference_modular(
        &mut self,
        inference_type: InferenceType,
        sketch: Sketch,
        use_static: bool,
        use_dynamic: bool,
    ) -> Result<InferenceResults, String> {
        self.start_time = Some(SystemTime::now());
        self.update_status(InferenceStatus::Started);

        let mut summary_msg = String::new();
        // boolean flag used to signal we reached 0 candidates and do not need to continue further
        let mut finished_early = false;

        /* >> STEP 1: process basic components of the sketch to be used */
        // this also does few input simplifications, like filtering out unused function symbols from the BN
        let bn = Self::extract_bn(&sketch)?;
        let static_props = process_static_props(&sketch, &bn)?;
        let dynamic_props = process_dynamic_props(&sketch)?;

        self.bn = Some(bn);
        self.static_props = Some(static_props);
        self.dynamic_props = Some(dynamic_props);
        self.update_status(InferenceStatus::ProcessedInputs);

        /* >> STEP 2: evaluation of static properties */

        if use_static && !finished_early {
            /* >> STEP 2A: make default symbolic transition graph for FOL evaluation */

            // select (can be random) a variable that will be used as a base for adding extra symbolic
            // variables (that we need to encode FOL vars)
            let base_var = self.bn()?.variables().collect::<Vec<_>>()[0];
            let base_var_name = self.bn()?.as_graph().get_variable_name(base_var).clone();

            self.graph = Some(prepare_graph_for_static_fol(
                self.bn()?,
                self.stat_props()?,
                &base_var_name,
                None,
            )?);
            self.update_status(InferenceStatus::GeneratedContextStatic);
            let msg = format!(
                "N. of candidates before evaluating any properties: {}\n",
                self.current_candidate_colors()?.exact_cardinality()
            );
            summary_msg.push_str(&msg);

            /* >> STEP 2B: actually evaluate static properties */
            self.eval_static(&base_var_name)?;
            let msg = format!(
                "N. of candidates after evaluating static props: {}\n",
                self.current_candidate_colors()?.exact_cardinality()
            );
            summary_msg.push_str(&msg);
        }
        finished_early = self.check_if_finished_unsat(false)?;

        /* >> STEP 3: evaluation of dynamic properties */
        if use_dynamic && !finished_early {
            /* >> STEP 3A: make symbolic transition graph for HCTL evaluation with restricted unit BDD */
            let old_unit_bdd = self.current_candidate_colors()?.into_bdd();
            let old_context = self.graph()?.symbolic_context();
            self.graph = Some(prepare_graph_for_dynamic_hctl(
                self.bn()?,
                self.dyn_props()?,
                Some((&old_unit_bdd, old_context)),
            )?);
            self.update_status(InferenceStatus::GeneratedContextDynamic);

            /* >> STEP 3B: actually evaluate dynamic properties */
            self.eval_dynamic()?;
            let msg = format!(
                "N. of candidates after evaluating dynamic props: {}\n",
                self.current_candidate_colors()?.approx_cardinality()
            );
            summary_msg.push_str(&msg);
        }

        /* >> STEP 4: process and save results */
        self.raw_sat_colors = Some(self.graph()?.mk_unit_colors());
        let num_sat_networks = self
            .final_sat_colors()?
            .exact_cardinality()
            .to_u128()
            .unwrap();

        if num_sat_networks == 0 {
            let msg = format!(
                "Sketch found unsatisfiable after processing {} static and {} dynamic properties\n",
                self.num_finished_stat_props(),
                self.num_finished_dyn_props()
            );
            summary_msg.push_str(&msg);
        } else {
            // let's convert all symbolic structs to the "pure" symbolic context (without any additional vars)
            // this is useful if we export the color BDD and want to reload it later
            let current_context: &biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext =
                self.graph()?.symbolic_context();
            let pure_context = current_context.as_canonical_context();
            let current_sat_colors = self.graph()?.mk_unit_colors();
            let current_unit_bdd = current_sat_colors.as_bdd();
            let pure_unit_bdd = pure_context
                .transfer_from(current_unit_bdd, current_context)
                .unwrap();
            let pure_sat_colors = GraphColors::new(pure_unit_bdd.clone(), &pure_context);
            let pure_graph =
                SymbolicAsyncGraph::with_custom_context(self.bn()?, pure_context, pure_unit_bdd)?;

            self.graph = Some(pure_graph);
            self.raw_sat_colors = Some(pure_sat_colors);
        }
        self.update_status(InferenceStatus::FinishedSuccessfully);

        let num_update_fns_per_var =
            num_update_fn_variants_per_var(self.final_sat_colors()?, self.bn()?);
        let total_time = self.total_duration().unwrap();
        let results = InferenceResults::new(
            inference_type,
            num_sat_networks,
            total_time,
            &summary_msg,
            self.status_updates.clone(),
            num_update_fns_per_var,
        );
        self.results = Some(results.clone());
        Ok(results)
    }
}

/// Check if InferenceStatus requires number of remaining candidates when reporting about
/// progress.
fn requires_candidate_num(status: &InferenceStatus) -> bool {
    !matches!(
        status,
        InferenceStatus::Created
            | InferenceStatus::Started
            | InferenceStatus::ProcessedInputs
            | InferenceStatus::EvaluatedAllStatic
            | InferenceStatus::EvaluatedAllDynamic
            | InferenceStatus::DetectedUnsat
            | InferenceStatus::Error
    )
}

pub(crate) fn track_progress(intermediate_result: &GraphColoredVertices, msg: &str) {
    println!(
        "Current BDD size: {};\tMessage: \"{msg}\"",
        intermediate_result.symbolic_size()
    );
}
