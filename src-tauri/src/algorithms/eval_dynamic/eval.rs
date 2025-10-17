use std::collections::HashMap;

use crate::algorithms::eval_dynamic::_attractors::sort_colors_by_attr_num;
use crate::algorithms::eval_dynamic::_fixed_points::colors_where_fixed_points;
use crate::algorithms::eval_dynamic::_trap_spaces::{
    colors_where_essential_traps, colors_where_minimal_traps,
};
use crate::algorithms::eval_dynamic::encode::encode_dataset_hctl_str;
use crate::algorithms::eval_dynamic::prepare_graph::get_ts_extended_symbolic_graph;
use crate::algorithms::eval_dynamic::processed_props::{DataEncodingType, ProcessedDynProp};
use biodivine_hctl_model_checker::model_checking::{
    _model_check_extended_formula_dirty, _model_check_formula_dirty,
};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};

use super::_trajectory::colors_with_trajectory;
use super::utils::transform_obs_to_vertex_set;

/// Model check a property and get colors for which the property holds universally
/// (in every state).
///
/// If we have an extended formula with wild-card propositions, arg `context_sets` should
/// give evaluation set for each such proposition.
fn model_check_colors_universal<F: FnMut(&GraphColoredVertices, &str)>(
    stg: &SymbolicAsyncGraph,
    formula: &str,
    context_sets: HashMap<String, GraphColoredVertices>,
    progress_callback: &mut F,
) -> Result<GraphColors, String> {
    // First, run model checking to compute all valid stat-color pairs
    // If there are no sub-properties, use standard universal HCTL model checking
    let mc_results = if context_sets.is_empty() {
        _model_check_formula_dirty(formula, stg, progress_callback)?
    } else {
        _model_check_extended_formula_dirty(formula, stg, &context_sets, progress_callback)?
    };

    // do universal projection on the colors of the given `colored_vertices`.
    let complement = stg.unit_colored_vertices().minus(&mc_results);
    let universal_colors = stg.unit_colors().minus(&complement.colors());
    Ok(universal_colors)
}

/// Evaluate given dynamic property given the symbolic transition graph.
pub fn eval_dyn_prop<F: FnMut(&GraphColoredVertices, &str)>(
    dyn_prop: &ProcessedDynProp,
    graph: &SymbolicAsyncGraph,
    progress_callback: &mut F,
) -> Result<GraphColors, String> {
    // use this set for initial progress callbacks before the computation starts
    let initial = graph.empty_colored_vertices();
    match &dyn_prop {
        ProcessedDynProp::ProcessedHctlFormula(prop) => {
            // First, handle all sub-properties coming from wild-card propositions
            // Each wild-card proposition gets a "context set" it will be evaluated as
            let mut context_sets: HashMap<String, GraphColoredVertices> = HashMap::new();
            for sub_prop in &prop.sub_properties {
                progress_callback(
                    initial,
                    &format!("Starting to evaluate sub-property {}.", sub_prop.id()),
                );

                // There is special type of "Observation" template which can only be used
                // as sub-property and has to be handled differently than the rest.
                if let ProcessedDynProp::ProcessedObservation(sub_prop_obs) = sub_prop {
                    let obs_context_set = transform_obs_to_vertex_set(
                        &sub_prop_obs.obs,
                        &sub_prop_obs.var_names,
                        graph,
                    )?;
                    context_sets.insert(sub_prop.id().to_string(), obs_context_set);
                } else {
                    // Otherwise it is a normal kind of property and we can handle it in a standard way
                    // Evaluation with [eval_dyn_prop] gives universal sat colors, we just convert the types
                    let sat_colors = eval_dyn_prop(sub_prop, graph, progress_callback)?;
                    let colored_vertices =
                        GraphColoredVertices::new(sat_colors.into_bdd(), graph.symbolic_context());
                    context_sets.insert(sub_prop.id().to_string(), colored_vertices);
                }
            }

            // use a version of HCTL model checking for extended formulae
            progress_callback(initial, "Starting computation using HCTL model checker.");
            model_check_colors_universal(graph, &prop.formula, context_sets, progress_callback)
        }
        ProcessedDynProp::ProcessedAttrCount(prop) => {
            // custom implementation (can be made more efficient if needed)
            // TODO: could be optimized by first computing fixed-points and removing colors where N_fp > MAX_ATTR

            // compute full attractors (on remaining colors) and get colors with correct n. of attrs
            let initial = graph.empty_colored_vertices();
            progress_callback(initial, "Starting attractor computation.");
            let colors_per_num_attrs: Vec<GraphColors> =
                sort_colors_by_attr_num(graph, progress_callback);
            let mut sat_colors = graph.mk_empty_colors();
            for (num_attrs, color_set) in colors_per_num_attrs.iter().enumerate() {
                if num_attrs >= prop.minimal && num_attrs <= prop.maximal {
                    sat_colors = sat_colors.union(color_set)
                }
            }
            Ok(sat_colors)
        }
        ProcessedDynProp::ProcessedTrapSpace(prop) => {
            // custom implementation (can definitely be made more efficient if needed)

            // get colors where all the observations are (general) trap spaces
            progress_callback(
                initial,
                "Starting computing trap spaces using model checker.",
            );
            let trap_space_formula =
                encode_dataset_hctl_str(&prop.dataset, None, DataEncodingType::TrapSpace)?;
            let mut sat_colors = model_check_colors_universal(
                graph,
                &trap_space_formula,
                HashMap::new(),
                progress_callback,
            )?;

            // if needed, restrict colors to only a set where the TSs are minimal or non-percolable
            if prop.minimal || prop.nonpercolable {
                // compute new trap-space context
                let bn = graph.as_network().unwrap();
                let unit_bdd = sat_colors.as_bdd();
                let original_context = graph.symbolic_context();
                let (space_ctx, space_graph) =
                    get_ts_extended_symbolic_graph(bn, Some((unit_bdd, original_context)))?;

                let observations = prop.dataset.observations().clone();
                let var_names = prop.dataset.variable_names();

                // note that all minimal TSs are non-percolable
                sat_colors = if prop.minimal {
                    progress_callback(initial, "Starting minimal trap spaces computation.");
                    colors_where_minimal_traps(observations, &var_names, &space_graph, &space_ctx)
                } else {
                    progress_callback(initial, "Starting essential trap spaces computation.");
                    colors_where_essential_traps(observations, &var_names, &space_graph, &space_ctx)
                };

                // switch color BDDs back to correct context
                let error_msg =
                    "Internal error during BDD transfer from one context to another.".to_string();
                let sat_colors_bdd = original_context
                    .transfer_from(sat_colors.as_bdd(), space_ctx.inner_context())
                    .ok_or(error_msg)?;
                sat_colors = GraphColors::new(sat_colors_bdd, original_context)
            }

            Ok(sat_colors)
        }
        ProcessedDynProp::ProcessedFixedPoint(prop) => {
            let observations = prop.dataset.observations().clone();
            let var_names = prop.dataset.variable_names();

            progress_callback(initial, "Starting to compute fixed points.");
            let sat_colors = colors_where_fixed_points(observations, &var_names, graph);
            Ok(sat_colors)
        }
        ProcessedDynProp::ProcessedSimpleTrajectory(prop) => {
            progress_callback(
                initial,
                "Starting to compute trajectory using reachability-based algorithm.",
            );
            colors_with_trajectory(&prop.dataset, graph, progress_callback)
        }
        ProcessedDynProp::ProcessedObservation(..) => {
            unreachable!("Observation cant be evaluated as a top-level property.")
        }
    }
}
