use crate::analysis::analysis_results::AnalysisResults;
use crate::sketchbook::properties::dynamic_props::DynPropertyType;
use crate::sketchbook::properties::{FirstOrderFormula, HctlFormula};
use crate::sketchbook::Sketch;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;
use std::time::SystemTime;

/// Status of the computation, together with a timestamp.
#[derive(Clone)]
pub enum InferenceStatus {
    Created(SystemTime),
    Started(SystemTime),
    ProcessedInputs(SystemTime),
    GeneratedGraph(SystemTime),
    EvaluatedStatic(SystemTime),
    EvaluatedDynamic(SystemTime),
    Finished(SystemTime),
    Error(SystemTime),
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
    graph: Option<SymbolicAsyncGraph>,
    /// Static properties converted to FO logic.
    static_props: Option<Vec<FirstOrderFormula>>,
    /// Dynamic properties converted to HCTL.
    dynamic_props: Option<Vec<HctlFormula>>,
    /// Intermediate set of candidate colors, gradually updated during computation.
    raw_intermediate_colors: Option<GraphColors>,
    /// Set of final satisfying colors (once computed).
    raw_sat_colors: Option<GraphColors>,
    /// Vector with all time-stamped status updates. The last is the latest status.
    status_updates: Vec<InferenceStatus>,
}

impl InferenceSolver {
    fn bn(&self) -> Result<&BooleanNetwork, String> {
        if let Some(bn) = &self.bn {
            Ok(bn)
        } else {
            Err("Boolean network not yet computed.".to_string())
        }
    }

    fn graph(&self) -> Result<&SymbolicAsyncGraph, String> {
        if let Some(graph) = &self.graph {
            Ok(graph)
        } else {
            Err("Transition graph not yet computed.".to_string())
        }
    }

    fn sat_colors(&self) -> Result<&GraphColors, String> {
        if let Some(colors) = &self.raw_sat_colors {
            Ok(colors)
        } else {
            Err("Satisfying colors not yet computed.".to_string())
        }
    }
}

impl InferenceSolver {
    /// Prepares new "empty" `InferenceSolver` instance.
    /// The computation is started by [start_computing] later.
    pub fn new() -> InferenceSolver {
        InferenceSolver {
            bn: None,
            graph: None,
            static_props: None,
            dynamic_props: None,
            raw_intermediate_colors: None,
            raw_sat_colors: None,
            status_updates: vec![InferenceStatus::Created(SystemTime::now())],
        }
    }

    /// Partially process the sketch into individual components.
    ///
    /// WARNING: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// static properties, all but generic dynamic properties.
    fn process_inputs_prototype(
        sketch: Sketch,
    ) -> Result<(BooleanNetwork, Vec<FirstOrderFormula>, Vec<HctlFormula>), String> {
        // todo: at the moment we just use HCTL dynamic properties, and no static properties
        let bn = sketch.model.to_bn();
        let static_properties = vec![];

        let mut dynamic_properties = vec![];
        for (_, dyn_prop) in sketch.properties.dyn_props() {
            if let DynPropertyType::GenericDynProp(prop) = dyn_prop.get_prop_data() {
                dynamic_properties.push(prop.clone().processed_formula)
            }
        }
        Ok((bn, static_properties, dynamic_properties))
    }

    /// Run the prototype version of the inference.
    /// This wraps the [run_computation_prototype_inner] to also log potential errors.
    ///
    /// WARNING: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// static properties, all but generic dynamic properties.
    pub fn run_computation_prototype(&mut self, sketch: Sketch) -> Result<AnalysisResults, String> {
        let results = self.run_computation_prototype_inner(sketch);
        if results.is_err() {
            self.status_updates
                .push(InferenceStatus::Error(SystemTime::now()));
        }
        results
    }

    /// Run the prototype version of the inference.
    ///
    /// WARNING: This is only a prototype, and considers just parts of the sketch that are easy to
    /// process at the moment. Some parts are lost, including "dual regulations", some kinds of
    /// static properties, all but generic dynamic properties.
    fn run_computation_prototype_inner(
        &mut self,
        sketch: Sketch,
    ) -> Result<AnalysisResults, String> {
        let start_time = SystemTime::now();
        self.status_updates
            .push(InferenceStatus::Started(start_time));

        // step 1: process basic components of the sketch to be used
        let (bn, static_props, dynamic_props) = Self::process_inputs_prototype(sketch)?;
        self.bn = Some(bn);
        self.static_props = Some(static_props);
        self.dynamic_props = Some(dynamic_props);
        self.status_updates
            .push(InferenceStatus::ProcessedInputs(SystemTime::now()));

        // step 2: make default symbolic transition graph
        self.graph = Some(SymbolicAsyncGraph::new(self.bn()?)?);
        self.raw_intermediate_colors = Some(self.graph()?.mk_unit_colors());
        self.status_updates
            .push(InferenceStatus::GeneratedGraph(SystemTime::now()));

        // step 3: todo: evaluate static properties, restrict colors
        self.status_updates
            .push(InferenceStatus::EvaluatedStatic(SystemTime::now()));

        // step 4: todo: evaluate dynamic properties, restrict colors
        self.status_updates
            .push(InferenceStatus::EvaluatedDynamic(SystemTime::now()));

        // step 5: process results, compute few statistics, return some results struct
        let finish_time = SystemTime::now();
        self.raw_sat_colors = self.raw_intermediate_colors.clone();
        self.status_updates
            .push(InferenceStatus::Finished(finish_time));

        let num_sat_networks = self.sat_colors()?.approx_cardinality() as u64;
        let total_time = finish_time.duration_since(start_time).unwrap();
        let results = AnalysisResults::new(num_sat_networks, total_time);
        Ok(results)
    }
}
