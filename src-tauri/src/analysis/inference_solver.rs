use crate::algorithms::eval_dynamic::template_eval::eval_dyn_prop;
use crate::algorithms::eval_static::template_eval::eval_static_prop;
use crate::analysis::analysis_results::AnalysisResults;
use crate::log;
use crate::sketchbook::properties::{DynProperty, StatProperty};
use crate::sketchbook::Sketch;
use biodivine_hctl_model_checker::mc_utils::get_extended_symbolic_graph;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::BooleanNetwork;
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
        // todo: at the moment we just use HCTL dynamic properties, and all static properties
        let bn = sketch.model.to_bn_with_plain_regulations();
        let mut static_properties = vec![];
        for (_, stat_prop) in sketch.properties.stat_props() {
            static_properties.push(stat_prop.clone());
        }

        let mut dynamic_properties = vec![];
        for (_, dyn_prop) in sketch.properties.dyn_props() {
            dynamic_properties.push(dyn_prop.clone());
        }
        Ok((bn, static_properties, dynamic_properties))
    }

    /// Evaluate previously collected static properties, and restrict the unit set of the
    /// graph to the set of valid colors.
    ///
    /// TODO: function `eval_static_prop` needs to be finished.
    fn eval_static(&mut self) -> Result<(), String> {
        for stat_property in self.stat_props()?.clone() {
            let inferred_colors = eval_static_prop(stat_property, self.bn()?, self.graph()?)?;
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
    fn run_whole_inference_prototype_inner(
        &mut self,
        sketch: Sketch,
    ) -> Result<AnalysisResults, String> {
        let mut metadata = String::new();
        self.status_update(InferenceStatus::Started);

        // step 1: process basic components of the sketch to be used
        let (bn, static_props, dynamic_props) = Self::extract_inputs(sketch)?;
        self.bn = Some(bn);
        self.static_props = Some(static_props);
        self.dynamic_props = Some(dynamic_props);
        self.status_update(InferenceStatus::ProcessedInputs);

        // step 2: todo: check how many HCTL propositions we need to eval the formulae
        let num_hctl_vars = 3;

        // step 3: make default symbolic transition graph
        self.graph = Some(get_extended_symbolic_graph(self.bn()?, num_hctl_vars)?);
        self.status_update(InferenceStatus::GeneratedGraph);
        let msg = format!(
            "N. of candidates before evaluating any properties: {}\n",
            self.current_candidate_colors()?.approx_cardinality()
        );
        metadata.push_str(&msg);

        // step 4: todo: evaluate static properties, restrict colors
        self.eval_static()?;
        let msg = format!(
            "N. of candidates after evaluating static props: {}\n",
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

        // step 5: process results, compute few statistics, return some results struct
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
