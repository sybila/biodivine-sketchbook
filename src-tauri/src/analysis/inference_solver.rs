use crate::algorithms::eval_dynamic::eval::eval_dyn_prop;
use crate::algorithms::eval_dynamic::prepare_graph::prepare_graph_for_dynamic;
use crate::algorithms::eval_static::eval::eval_static_prop;
use crate::algorithms::eval_static::prepare_graph::prepare_graph_for_static;
use crate::analysis::analysis_results::AnalysisResults;
use crate::analysis::analysis_type::AnalysisType;
use crate::log;
use crate::sketchbook::properties::{DynProperty, StatProperty};
use crate::sketchbook::Sketch;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::BooleanNetwork;
use num_bigint::BigInt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tauri::async_runtime::RwLock;

/// Status of the computation, together with a timestamp.
#[derive(Clone, Debug)]
pub enum InferenceStatus {
    Created,
    Started,
    ProcessedInputs,
    GeneratedGraph,
    EvaluatedStatic,
    EvaluatedDynamic,
    Finished,
    Error,
}

/// Object encompassing the process of the full BN inference.
///
/// It tracks the intermediate results and low-level structures, and it provides hooks to the
/// actual algorithms.
///
/// By tracking the intermediate results, we may be able to observe the computation, and potentially
/// "fork" or re-use parts of the computation in future. For now we do not run the computation
/// itself on a special thread, but in future, we may.
#[derive(Clone)]
pub struct InferenceSolver {
    /// Boolean Network instance.
    bn: Option<BooleanNetwork>,
    /// Symbolic transition graph for the system.
    /// Its set of unit colors is gradually updated during computation, and it consists of
    /// currently remaining valid candidate colors.
    graph: Option<SymbolicAsyncGraph>,
    /// Static properties.
    static_props: Option<Vec<StatProperty>>,
    /// Dynamic properties.
    dynamic_props: Option<Vec<DynProperty>>,
    /// Set of final satisfying colors (once computed).
    raw_sat_colors: Option<GraphColors>,
    /// Vector with all time-stamped status updates. The last is the latest status.
    status_updates: Vec<(InferenceStatus, SystemTime)>,
    /// Flag to signal cancellation
    should_stop: Arc<AtomicBool>,
    /// Processed results.
    results: Option<AnalysisResults>,
}

impl InferenceSolver {
    /// Prepares new "empty" `InferenceSolver` instance.
    /// The computation is started by one of the `run_<algorithm>` methods later.
    pub fn new() -> InferenceSolver {
        InferenceSolver {
            bn: None,
            graph: None,
            static_props: None,
            dynamic_props: None,
            raw_sat_colors: None,
            status_updates: vec![(InferenceStatus::Created, SystemTime::now())],
            should_stop: Arc::new(AtomicBool::new(false)),
            results: None,
        }
    }

    /// Reference getter for a Boolean network.
    pub fn bn(&self) -> Result<&BooleanNetwork, String> {
        if let Some(bn) = &self.bn {
            Ok(bn)
        } else {
            Err("Boolean network not yet computed.".to_string())
        }
    }

    /// Reference getter for a transition graph.
    pub fn graph(&self) -> Result<&SymbolicAsyncGraph, String> {
        if let Some(graph) = &self.graph {
            Ok(graph)
        } else {
            Err("Transition graph not yet computed.".to_string())
        }
    }

    /// Reference getter for a vector of formulas for static properties.
    pub fn stat_props(&self) -> Result<&Vec<StatProperty>, String> {
        if let Some(stat_props) = &self.static_props {
            Ok(stat_props)
        } else {
            Err("Static properties not yet processed.".to_string())
        }
    }

    /// Reference getter for a vector of formulas for dynamic properties.
    pub fn dyn_props(&self) -> Result<&Vec<DynProperty>, String> {
        if let Some(dyn_props) = &self.dynamic_props {
            Ok(dyn_props)
        } else {
            Err("Dynamic properties not yet processed.".to_string())
        }
    }

    /// Reference getter for a set with satisfying graph colors.
    pub fn sat_colors(&self) -> Result<&GraphColors, String> {
        if let Some(colors) = &self.raw_sat_colors {
            Ok(colors)
        } else {
            Err("Satisfying colors not yet computed.".to_string())
        }
    }

    /// Get a current set of valid candidate colors.
    /// This is gradually updated during computation.
    pub fn current_candidate_colors(&self) -> Result<GraphColors, String> {
        Ok(self.graph()?.mk_unit_colors())
    }

    /// Get processed results if computed.
    pub fn results(&self) -> Result<&AnalysisResults, String> {
        if let Some(results) = &self.results {
            Ok(results)
        } else {
            Err("Processed results not yet computed.".to_string())
        }
    }

    fn update_status(&mut self, status: InferenceStatus) {
        let now = SystemTime::now();
        self.status_updates.push((status.clone(), now));

        let start_time = self.start_time().unwrap();
        let duration_since_start = now.duration_since(start_time).unwrap();
        log!(
            "0",
            "Inference status update: {:?}, {}s",
            status,
            duration_since_start.as_secs()
        );
    }

    /// Getter for a start time of the actual computation.
    pub fn start_time(&self) -> Result<SystemTime, String> {
        if self.status_updates.len() > 1 {
            Ok(self.status_updates[1].1)
        } else {
            Err("Computation not yet started.".to_string())
        }
    }

    /// Getter for a finish time of the actual computation.
    pub fn finish_time(&self) -> Result<SystemTime, String> {
        // there is always some status (since one is given during initialization)
        let (last_status, last_time) = self.status_updates.last().unwrap();
        if let InferenceStatus::Finished = last_status {
            Ok(*last_time)
        } else if let InferenceStatus::Error = last_status {
            Err("Computation failed to finish.".to_string())
        } else {
            Err("Computation not yet finished.".to_string())
        }
    }

    /// Set the cancellation flag. The actual cancellation does not happen immediately,
    /// we currently only allow cancelling only at certain checkpoints during computation.
    pub fn cancel(&self) {
        //debug!("`InferenceSolver` has received cancellation flag.");
        self.should_stop.store(true, Ordering::SeqCst);
    }

    /// Utility to check whether the cancellation flag was set. If it is set, the function
    /// returns error. Otherwise, nothing happens.
    fn check_cancellation(&self) -> Result<(), String> {
        if self.should_stop.load(Ordering::SeqCst) {
            return Err("Computation was cancelled.".to_string());
        }
        Ok(())
    }
}

/// Computation-related methods.
impl InferenceSolver {
    /// Run the prototype version of the inference using the given solver.
    /// This wraps the [run_inference_modular] to also log potential errors.
    ///
    /// The argument `analysis_type` specifies which kind of inference should be used.
    /// Currently, we support full inference with all properties, and partial inferences with only
    /// static or only dynamic properties.
    ///
    /// The results are saved to sepcific fields of the provided solver and can be retrieved later.
    /// They are also returned, which is now used for logging later.
    ///
    /// TODO: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// static properties, all but generic dynamic properties.
    pub async fn run_inference_async(
        solver: Arc<RwLock<InferenceSolver>>,
        sketch: Sketch,
        analysis_type: AnalysisType,
    ) -> Result<AnalysisResults, String> {
        {
            let solver = solver.read().await;
            solver.check_cancellation()?; // Early check before starting
        }

        /* todo: currently, we use this "write lock" to lock the solver for the whole inference
         * This makes it impossible to get quick lock for cancellation or intermediate result fetch
         * We should make the inference modular in an async sense, so that:
         *  1) fetching intermediate results is possible
         *  2) proper cancellation is possible
         */

        let mut solver_write = solver.write().await;
        let results = match analysis_type {
            AnalysisType::Inference => {
                solver_write.run_inference_modular(analysis_type, sketch, true, true)
            }
            AnalysisType::StaticCheck => {
                solver_write.run_inference_modular(analysis_type, sketch, true, false)
            }
            AnalysisType::DynamicCheck => {
                solver_write.run_inference_modular(analysis_type, sketch, false, true)
            }
        };
        if results.is_err() {
            solver_write.update_status(InferenceStatus::Error);
        }

        // now lets just drop the lock to allow for potential cancellation (the cancellation thread must
        // achieve a lock) and we can at least check if computation should have been cancelled...
        // (this is currently just a placeholder check before we do it properly during computation)
        drop(solver_write);
        thread::sleep(Duration::from_millis(10));

        {
            let solver = solver.read().await;
            solver.check_cancellation()?; // Check if computation should have been cancelled
        }

        results
    }

    /// Extract and convert relevant components from the sketch (boolean network, properties).
    fn extract_inputs(
        sketch: Sketch,
    ) -> Result<(BooleanNetwork, Vec<StatProperty>, Vec<DynProperty>), String> {
        // todo: at the moment we just use HCTL dynamic properties, and automatically generated regulation properties

        let bn = sketch.model.to_bn_with_plain_regulations(None);
        // remove all unused function symbols, as these would cause problems later
        let bn = bn.prune_unused_parameters();

        let mut static_props = vec![];
        for (id, stat_prop) in sketch.properties.stat_props() {
            static_props.push((id, stat_prop.clone()));
        }

        let mut dynamic_props = vec![];
        for (id, dyn_prop) in sketch.properties.dyn_props() {
            dynamic_props.push((id, dyn_prop.clone()));
        }

        // sort properties by IDs for deterministic computation times (and remove the IDs)
        dynamic_props.sort_by(|(a_id, _), (b_id, _)| a_id.cmp(b_id));
        let dynamic_props = dynamic_props.into_iter().map(|p| p.1).collect();
        static_props.sort_by(|(a_id, _), (b_id, _)| a_id.cmp(b_id));
        let static_props = static_props.into_iter().map(|p| p.1).collect();

        Ok((bn, static_props, dynamic_props))
    }

    /// Evaluate previously collected static properties, and restrict the unit set of the
    /// graph to the set of valid colors.
    ///
    /// TODO: function `eval_static_prop` needs to be finished.
    fn eval_static(&mut self, base_var_name: &str) -> Result<(), String> {
        for stat_property in self.stat_props()?.clone() {
            self.check_cancellation()?; // check if cancellation flag was set during computation

            let inferred_colors =
                eval_static_prop(stat_property, self.bn()?, self.graph()?, base_var_name)?;
            let colored_vertices = GraphColoredVertices::new(
                inferred_colors.into_bdd(),
                self.graph()?.symbolic_context(),
            );
            let new_graph: SymbolicAsyncGraph = self.graph()?.restrict(&colored_vertices);
            self.graph = Some(new_graph);
        }
        self.update_status(InferenceStatus::EvaluatedStatic);
        Ok(())
    }

    /// Evaluate previously collected dynamic properties, and restrict the unit set of the
    /// graph to the set of valid colors.
    ///
    /// TODO: function `eval_dyn_prop` needs to be finished.
    fn eval_dynamic(&mut self) -> Result<(), String> {
        for dyn_property in self.dyn_props()?.clone() {
            self.check_cancellation()?; // check if cancellation flag was set during computation

            let inferred_colors = eval_dyn_prop(dyn_property, self.graph()?)?;
            let colored_vertices = GraphColoredVertices::new(
                inferred_colors.into_bdd(),
                self.graph()?.symbolic_context(),
            );
            let new_graph: SymbolicAsyncGraph = self.graph()?.restrict(&colored_vertices);
            self.graph = Some(new_graph);
        }
        self.update_status(InferenceStatus::EvaluatedDynamic);
        Ok(())
    }

    /// Internal modular variant of the inference. You can choose which parts to select.
    /// For example, you can only consider static properties, only dynamic properties, or all.
    ///
    /// TODO: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// static properties, all but generic dynamic properties.
    fn run_inference_modular(
        &mut self,
        analysis_type: AnalysisType,
        sketch: Sketch,
        use_static: bool,
        use_dynamic: bool,
    ) -> Result<AnalysisResults, String> {
        self.update_status(InferenceStatus::Started);

        let mut metadata = String::new();
        // boolean flag used to signal we reached 0 candidates and do not need to continue further
        let mut finished_early = false;

        /* >> STEP 1: process basic components of the sketch to be used */
        // this also does few input simplifications, like filtering out unused function symbols from the BN
        let (bn, static_props, dynamic_props) = Self::extract_inputs(sketch)?;

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

            self.graph = Some(prepare_graph_for_static(
                self.bn()?,
                self.stat_props()?,
                &base_var_name,
                None,
            )?);
            self.update_status(InferenceStatus::GeneratedGraph);
            let msg = format!(
                "N. of candidates before evaluating any properties: {}\n",
                self.current_candidate_colors()?.approx_cardinality()
            );
            metadata.push_str(&msg);

            /* >> STEP 2B: actually evaluate static properties */
            self.eval_static(&base_var_name)?;
            let msg = format!(
                "N. of candidates after evaluating static props: {}\n",
                self.current_candidate_colors()?.approx_cardinality()
            );
            metadata.push_str(&msg);
        }
        finished_early = self.check_if_finished()?;

        /* >> STEP 3: evaluation of dynamic properties */
        if use_dynamic && !finished_early {
            /* >> STEP 3A: make symbolic transition graph for HCTL evaluation with restricted unit BDD */
            let old_unit_bdd = self.current_candidate_colors()?.into_bdd();
            let old_context = self.graph()?.symbolic_context();
            self.graph = Some(prepare_graph_for_dynamic(
                self.bn()?,
                self.dyn_props()?,
                Some((&old_unit_bdd, old_context)),
            )?);
            self.update_status(InferenceStatus::GeneratedGraph);
            //let msg = format!(
            //    "N. of candidates before evaluating any dynamic properties: {}\n",
            //    self.current_candidate_colors()?.approx_cardinality()
            //);
            //metadata.push_str(&msg);

            /* >> STEP 3B: actually evaluate dynamic properties */
            self.eval_dynamic()?;
            let msg = format!(
                "N. of candidates after evaluating dynamic props: {}\n",
                self.current_candidate_colors()?.approx_cardinality()
            );
            metadata.push_str(&msg);
        }

        /* >> STEP 4: save results */
        self.raw_sat_colors = Some(self.graph()?.mk_unit_colors());
        self.update_status(InferenceStatus::Finished);

        let num_sat_networks = self.sat_colors()?.approx_cardinality() as u64;
        let total_time = self
            .finish_time()?
            .duration_since(self.start_time()?)
            .unwrap();
        let results = AnalysisResults::new(analysis_type, num_sat_networks, total_time, &metadata);
        self.results = Some(results.clone());
        Ok(results)
    }

    /// Check if we already reached 0 candidates and do not need to continue further.
    fn check_if_finished(&self) -> Result<bool, String> {
        Ok(self.current_candidate_colors()?.exact_cardinality() == BigInt::from(0))
    }
}
