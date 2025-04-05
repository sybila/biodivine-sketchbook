use crate::algorithms::eval_dynamic::saturated_reachability::{reach_bwd, reachability_step};
use crate::algorithms::eval_dynamic::utils::transform_obs_to_singleton_vertex;
use crate::sketchbook::observations::Dataset;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::VariableId;

/// Compute colors where the whole `target` is back-reachable from `initial` inside `universe`.
/// When target is a single state, this is simple. When target consists of multiple states,
/// all of them must be reachable.
pub fn colors_where_target_bwd_reachable<F: FnMut(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
    target: &GraphColoredVertices,
    universe: &GraphColoredVertices,
    variables: &[VariableId],
    progress_callback: &mut F,
) -> GraphColors {
    let mut bwd_reached_set = initial.intersect(universe);
    let target = target.intersect(universe);
    let starting_colors = universe.colors();
    let mut colors_not_finished = starting_colors.clone();

    loop {
        progress_callback(
            &bwd_reached_set,
            "Computing reachability set using saturation.",
        );

        // colors finished in previous step
        let colors_finished = universe.minus_colors(&colors_not_finished).colors();

        // remove newly finished colors from the universe set for remaining computation
        let universe = universe.minus_colors(&colors_finished);

        if reachability_step(&mut bwd_reached_set, &universe, variables, |v, s| {
            graph.var_pre(v, s)
        }) {
            break;
        }

        // update color set remaining for computation (colors for which target is not fully reached yet)
        colors_not_finished = target.minus(&bwd_reached_set).colors();
    }
    // return initial color set minus all colors for which the reachability successfully finished
    starting_colors.minus(&colors_not_finished)
}

/// Compute colors where there is a trajectory between the given list of states (each state encoded
/// as a logical conjunction). The trajectory must start at the first state, go through all successive
/// states in order, and reach the final state.
pub fn colors_with_trajectory<F: FnMut(&GraphColoredVertices, &str)>(
    dataset: &Dataset,
    graph: &SymbolicAsyncGraph,
    progress_callback: &mut F,
) -> Result<GraphColors, String> {
    assert!(dataset.num_observations() > 0);
    let observations = dataset.observations();
    let var_names = dataset.variable_names();
    let mut trajectory_states: Vec<GraphColoredVertices> = observations
        .iter()
        .map(|obs| transform_obs_to_singleton_vertex(obs, &var_names, graph).unwrap())
        .collect();

    // first pre-compute a subset of colours isn which final state is reachable from all other trajectory states
    let last_state = trajectory_states.pop().unwrap();
    let variables = graph.variables().collect::<Vec<_>>();
    let universe = graph.unit_colored_vertices();
    let msg = "Pre-computing backward reachability from the last observation state.";
    progress_callback(universe, msg);

    let bwd_reach_last = reach_bwd(graph, &last_state, universe, &variables, progress_callback);
    let mut sat_colors = graph.mk_unit_colors();
    for trajectory_state in &trajectory_states {
        // restrict a subset of colours where this state lays in bwd_reach_last
        let colors_to_keep = bwd_reach_last.intersect(trajectory_state).colors();
        sat_colors = sat_colors.intersect(&colors_to_keep);
    }
    let msg = format!(
        "After pre-pruning, {} candidates remain.",
        sat_colors.approx_cardinality()
    );
    progress_callback(universe, &msg);

    // one-by-one, check reachability between successive pairs of states (starting from the last one)
    // at each iteration, get rid of non-satisfying colors, simplifying further computation
    // note that the last state was already handled before
    let mut to_state = trajectory_states.pop().unwrap();
    for (index, from_state) in trajectory_states.into_iter().enumerate().rev() {
        let universe = graph.unit_colored_vertices().intersect_colors(&sat_colors);
        let msg = format!("Computing reachability from state n.{index}.");
        progress_callback(&universe, &msg);

        //let bwd_reach_to_state = reach_bwd(graph, &to_state, &universe, &variables);
        //let colors_to_keep = bwd_reach_to_state.intersect(&from_state).colors();
        //sat_colors = sat_colors.intersect(&colors_to_keep);

        sat_colors = colors_where_target_bwd_reachable(
            graph,
            &to_state,
            &from_state,
            &universe,
            &variables,
            progress_callback,
        );
        to_state = from_state;
    }
    Ok(sat_colors)
}
