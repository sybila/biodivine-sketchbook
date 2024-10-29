use crate::algorithms::eval_dynamic::_attractors::sort_colors_by_attr_num;
use crate::algorithms::eval_dynamic::_trap_spaces::{
    colors_where_essential_traps, colors_where_minimal_traps,
};
use crate::algorithms::eval_dynamic::encode::encode_dataset_hctl_str;
use crate::algorithms::eval_dynamic::prepare_graph::get_ts_extended_symbolic_graph;
use crate::algorithms::eval_dynamic::processed_props::{DataEncodingType, ProcessedDynProp};
use biodivine_hctl_model_checker::model_checking::model_check_formula_dirty;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};

/// Evaluate given dynamic property given the symbolic transition graph.
pub fn eval_dyn_prop(
    dyn_prop: ProcessedDynProp,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColors, String> {
    match &dyn_prop {
        ProcessedDynProp::ProcessedHctlFormula(prop) => {
            // use HCTL model checking
            let mc_results = model_check_formula_dirty(&prop.formula, graph)?;
            Ok(mc_results.colors())
        }
        ProcessedDynProp::ProcessedAttrCount(prop) => {
            // custom implementation (can be made more efficient if needed)
            // todo: optimize - first just compute fixed-points and get colors where N_fp <= MAX_ATTR

            // compute full attractors (on remaining colors) and get colors with correct n. of attrs
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
            let trap_space_formula =
                encode_dataset_hctl_str(&prop.dataset, None, DataEncodingType::TrapSpace)?;
            let mc_results = model_check_formula_dirty(&trap_space_formula, graph)?;
            let mut sat_colors = mc_results.colors();

            // TODO: if needed, restrict colors to only a set where the TSs are minimal or non-percolable
            if prop.minimal || prop.nonpercolable {
                // compute new trap-space context
                let bn = graph.as_network().unwrap();
                let unit_bdd = sat_colors.as_bdd();
                let original_context = graph.symbolic_context();
                let (space_ctx, space_graph) =
                    get_ts_extended_symbolic_graph(bn, Some((&unit_bdd, &original_context)))?;

                let observations = prop.dataset.observations().clone();
                let var_names = prop.dataset.variable_names();

                // note that all minimal TSs are non-percolable
                sat_colors = if prop.minimal {
                    colors_where_minimal_traps(observations, &var_names, &space_graph, &space_ctx)
                } else {
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
    }
}
