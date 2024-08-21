use crate::algorithms::eval_dynamic::eval::eval_dyn_prop;
use crate::algorithms::eval_dynamic::prepare_graph::prepare_graph_for_dynamic;
use crate::algorithms::eval_static::eval::eval_static_prop;
use crate::algorithms::eval_static::prepare_graph::prepare_graph_for_static;
use crate::analysis::analysis_results::AnalysisResults;
use crate::log;
use crate::sketchbook::properties::{DynProperty, StatProperty};
use crate::sketchbook::Sketch;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::BooleanNetwork;
use num_bigint::BigInt;
use std::time::SystemTime;

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

    fn status_update(&mut self, status: InferenceStatus) {
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
}

/// Computation-related methods.
impl InferenceSolver {
    /// Run the prototype version of the inference.
    /// This wraps the [run_inference_modular] to also log potential errors.
    ///
    /// TODO: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// static properties, all but generic dynamic properties.
    pub fn run_whole_inference(
        &mut self,
        sketch: Sketch,
    ) -> Result<AnalysisResults, String> {
        let results = self.run_inference_modular(sketch, true, true);
        if results.is_err() {
            self.status_update(InferenceStatus::Error);
        }
        results
    }

    /// Run the prototype version of the inference with static properties ONLY.
    /// This wraps the [run_inference_modular] to also log potential errors.
    ///
    /// TODO: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// static properties.
    pub fn run_partial_inference_static(
        &mut self,
        sketch: Sketch,
    ) -> Result<AnalysisResults, String> {
        let results = self.run_inference_modular(sketch, true, false);
        if results.is_err() {
            self.status_update(InferenceStatus::Error);
        }
        results
    }

    /// Run the prototype version of the inference with dynamic properties ONLY.
    /// This wraps the [run_inference_modular] to also log potential errors.
    ///
    /// TODO: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// dynamic properties.
    pub fn run_partial_inference_dynamic(
        &mut self,
        sketch: Sketch,
    ) -> Result<AnalysisResults, String> {
        let results = self.run_inference_modular(sketch, false, true);
        if results.is_err() {
            self.status_update(InferenceStatus::Error);
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
            let inferred_colors =
                eval_static_prop(stat_property, self.bn()?, self.graph()?, base_var_name)?;
            let colored_vertices = GraphColoredVertices::new(
                inferred_colors.into_bdd(),
                self.graph()?.symbolic_context(),
            );
            let new_graph: SymbolicAsyncGraph = self.graph()?.restrict(&colored_vertices);
            self.graph = Some(new_graph);
        }
        self.status_update(InferenceStatus::EvaluatedStatic);
        Ok(())
    }

    /// Evaluate previously collected dynamic properties, and restrict the unit set of the
    /// graph to the set of valid colors.
    ///
    /// TODO: function `eval_dyn_prop` needs to be finished.
    fn eval_dynamic(&mut self) -> Result<(), String> {
        for dyn_property in self.dyn_props()?.clone() {
            let inferred_colors = eval_dyn_prop(dyn_property, self.graph()?)?;
            let colored_vertices = GraphColoredVertices::new(
                inferred_colors.into_bdd(),
                self.graph()?.symbolic_context(),
            );
            let new_graph: SymbolicAsyncGraph = self.graph()?.restrict(&colored_vertices);
            self.graph = Some(new_graph);
        }
        self.status_update(InferenceStatus::EvaluatedDynamic);
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
        sketch: Sketch,
        use_static: bool,
        use_dynamic: bool,
    ) -> Result<AnalysisResults, String> {
        self.status_update(InferenceStatus::Started);

        let mut metadata = String::new();
        // boolean flag used to signal we reached 0 candidates and do not need to continue further
        let mut finished_early = false; 

        /* >> STEP 1: process basic components of the sketch to be used */
        // this also does few input simplifications, like filtering out unused function symbols from the BN 
        let (bn, static_props, dynamic_props) = Self::extract_inputs(sketch)?;

        self.bn = Some(bn);
        self.static_props = Some(static_props);
        self.dynamic_props = Some(dynamic_props);
        self.status_update(InferenceStatus::ProcessedInputs);

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
            self.status_update(InferenceStatus::GeneratedGraph);
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
            self.status_update(InferenceStatus::GeneratedGraph);
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

        /* >> STEP 4: process results, compute few statistics, return some results struct */
        self.raw_sat_colors = Some(self.graph()?.mk_unit_colors());
        self.status_update(InferenceStatus::Finished);

        let num_sat_networks = self.sat_colors()?.approx_cardinality() as u64;
        let total_time = self
            .finish_time()?
            .duration_since(self.start_time()?)
            .unwrap();
        let results = AnalysisResults::new(num_sat_networks, total_time, &metadata);
        Ok(results)
    }
    
    /// Check if we already reached 0 candidates and do not need to continue further.
    fn check_if_finished(&self) -> Result<bool, String> {
        Ok(self.current_candidate_colors()?.exact_cardinality() == BigInt::from(0))
    }
}
