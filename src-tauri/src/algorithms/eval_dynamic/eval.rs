use crate::algorithms::eval_dynamic::_attractors::sort_colors_by_attr_num;
use crate::algorithms::eval_dynamic::_trap_spaces::{
    colors_where_essential_traps, colors_where_minimal_traps,
};
use crate::algorithms::eval_dynamic::encode::encode_dataset_hctl_str;
use crate::algorithms::eval_dynamic::prepare_graph::get_ts_extended_symbolic_graph;
use crate::algorithms::eval_dynamic::processed_props::{DataEncodingType, ProcessedDynProp};
use biodivine_hctl_model_checker::model_checking::_model_check_formula_dirty;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};

use super::_trajectory::colors_with_trajectory;

/// Model check a property and get colors for which the property holds universally (in every state).
fn model_check_colors_universal<F: FnMut(&GraphColoredVertices, &str)>(
    stg: &SymbolicAsyncGraph,
    formula: &str,
    progress_callback: &mut F,
) -> Result<GraphColors, String> {
    // run model checking to compute all valid stat-color pairs
    let mc_results = _model_check_formula_dirty(formula, stg, progress_callback)?;
    // do universal projection on the colors of the given `colored_vertices`.
    let complement = stg.unit_colored_vertices().minus(&mc_results);
    let universal_colors = stg.unit_colors().minus(&complement.colors());
    Ok(universal_colors)
}

/// Evaluate given dynamic property given the symbolic transition graph.
pub fn eval_dyn_prop<F: FnMut(&GraphColoredVertices, &str)>(
    dyn_prop: ProcessedDynProp,
    graph: &SymbolicAsyncGraph,
    progress_callback: &mut F,
) -> Result<GraphColors, String> {
    match &dyn_prop {
        ProcessedDynProp::ProcessedHctlFormula(prop) => {
            // use HCTL model checking
            progress_callback(
                graph.empty_colored_vertices(),
                "Starting computation with HCTL model checker.",
            );
            model_check_colors_universal(graph, &prop.formula, progress_callback)
        }
        ProcessedDynProp::ProcessedAttrCount(prop) => {
            // custom implementation (can be made more efficient if needed)
            // TODO: could be optimized by first computing fixed-points and removing colors where N_fp > MAX_ATTR

            // compute full attractors (on remaining colors) and get colors with correct n. of attrs
            progress_callback(
                graph.empty_colored_vertices(),
                "Starting attractor computation using AEON-based algorithms.",
            );
            let colors_per_num_attrs: Vec<GraphColors> = sort_colors_by_attr_num(graph);
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
                graph.empty_colored_vertices(),
                "Starting trap space computation with HCTL model checker.",
            );
            let trap_space_formula =
                encode_dataset_hctl_str(&prop.dataset, None, DataEncodingType::TrapSpace)?;
            let mut sat_colors =
                model_check_colors_universal(graph, &trap_space_formula, progress_callback)?;

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
                    progress_callback(
                        graph.empty_colored_vertices(),
                        "Starting minimal trap spaces computation.",
                    );
                    colors_where_minimal_traps(observations, &var_names, &space_graph, &space_ctx)
                } else {
                    progress_callback(
                        graph.empty_colored_vertices(),
                        "Starting essential trap spaces computation.",
                    );
                    colors_where_essential_traps(observations, &var_names, &space_graph, &space_ctx)
                };

                // switch color BDDs back to correct context
                let sat_colors_bdd = original_context
                    .transfer_from(sat_colors.as_bdd(), space_ctx.inner_context())
                    .ok_or(
                        "Internal error during BDD transfer from one context to another."
                            .to_string(),
                    )?;
                sat_colors = GraphColors::new(sat_colors_bdd, original_context)
            }

            Ok(sat_colors)
        }
        ProcessedDynProp::ProcessedSimpleTrajectory(prop) => {
            progress_callback(
                graph.empty_colored_vertices(),
                "Starting to compute trajectory using reachability-based algorithm.",
            );
            colors_with_trajectory(&prop.dataset, graph, progress_callback)
        }
    }
}
