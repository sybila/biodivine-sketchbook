use crate::algorithms::eval_dynamic::template_eval::eval_dyn_prop;
use crate::algorithms::eval_static::template_eval::eval_static_prop;
use crate::analysis::analysis_results::AnalysisResults;
use crate::analysis::context_utils::{
    get_fol_extended_symbolic_graph, get_hctl_extended_symbolic_graph,
};
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
    /// This wraps the [run_computation_prototype_inner] to also log potential errors.
    ///
    /// WARNING: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// static properties, all but generic dynamic properties.
    pub fn run_whole_inference_prototype(
        &mut self,
        sketch: Sketch,
    ) -> Result<AnalysisResults, String> {
        let results = self.run_whole_inference_prototype_inner(sketch);
        if results.is_err() {
            self.status_update(InferenceStatus::Error);
        }
        results
    }

    /// Extract and convert relevant components from the sketch (boolean network, properties).
    fn extract_inputs(
        sketch: Sketch,
    ) -> Result<(BooleanNetwork, Vec<StatProperty>, Vec<DynProperty>), String> {
        // todo: at the moment we just use HCTL dynamic properties, and automatic regulation static properties

        let bn = sketch.model.to_bn_with_plain_regulations(None);
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

    /// Run the prototype version of the inference.
    ///
    /// WARNING: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// static properties, all but generic dynamic properties.
    ///
    /// TODO: for each analysis (HCTL, FOL, ..), create its own symbolic context
    fn run_whole_inference_prototype_inner(
        &mut self,
        sketch: Sketch,
    ) -> Result<AnalysisResults, String> {
        let mut metadata = String::new();
        self.status_update(InferenceStatus::Started);

        // step 1: process basic components of the sketch to be used
        let (bn, static_props, dynamic_props) = Self::extract_inputs(sketch)?;

        // todo: check how many extra HCTL and FOL vars we need to eval the properties
        // todo: do this inside of the evaluation sub-functions, make separate contexts for each
        // next lines is just explicit hack for now
        let num_hctl_vars = 3;
        let num_fol_vars = 3;
        let base_var = bn.variables().collect::<Vec<_>>()[0];
        let base_var_name = bn.as_graph().get_variable_name(base_var).clone();

        self.bn = Some(bn);
        self.static_props = Some(static_props);
        self.dynamic_props = Some(dynamic_props);
        self.status_update(InferenceStatus::ProcessedInputs);

        // step 2: make default symbolic transition graph for FOL evaluation
        self.graph = Some(get_fol_extended_symbolic_graph(
            self.bn()?,
            num_fol_vars,
            &base_var_name,
            None,
        )?);
        self.status_update(InferenceStatus::GeneratedGraph);
        let msg = format!(
            "N. of candidates before evaluating any static properties: {}\n",
            self.current_candidate_colors()?.approx_cardinality()
        );
        metadata.push_str(&msg);

        // step 3: evaluate static properties
        self.eval_static(&base_var_name)?;
        let msg = format!(
            "N. of candidates after evaluating static props: {}\n",
            self.current_candidate_colors()?.approx_cardinality()
        );
        metadata.push_str(&msg);

        // explicit check if we finished early
        if self.current_candidate_colors()?.exact_cardinality() == BigInt::from(0) {
            self.raw_sat_colors = Some(self.current_candidate_colors()?.clone());
            self.status_update(InferenceStatus::Finished);

            let num_sat_networks = 0u64;
            let total_time: std::time::Duration = self
                .finish_time()?
                .duration_since(self.start_time()?)
                .unwrap();
            let results = AnalysisResults::new(num_sat_networks, total_time, &metadata);
            return Ok(results);
        }

        // step 4: make symbolic transition graph for HCTL evaluation with restricted unit BDD
        let old_unit_bdd = self.current_candidate_colors()?.into_bdd();
        let old_context = self.graph()?.symbolic_context();
        self.graph = Some(get_hctl_extended_symbolic_graph(
            self.bn()?,
            num_hctl_vars,
            Some((&old_unit_bdd, old_context)),
        )?);
        self.status_update(InferenceStatus::GeneratedGraph);
        let msg = format!(
            "N. of candidates before evaluating any dynamic properties: {}\n",
            self.current_candidate_colors()?.approx_cardinality()
        );
        metadata.push_str(&msg);

        // step 5: evaluate dynamic properties
        self.eval_dynamic()?;
        let msg = format!(
            "N. of candidates after evaluating dynamic props: {}\n",
            self.current_candidate_colors()?.approx_cardinality()
        );
        metadata.push_str(&msg);

        // step 6: process results, compute few statistics, return some results struct
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
}
