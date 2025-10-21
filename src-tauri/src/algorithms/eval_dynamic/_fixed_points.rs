use crate::algorithms::eval_dynamic::utils::transform_obs_to_vertex_set;
use crate::sketchbook::observations::Observation;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::fixed_points::FixedPoints;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};

/// Compute colors where each given observation corresponds to a fixed point.
///
/// If the observation is fully specified, it corresponds to a sinlge state that
/// must be a fixed point. If it is partially specified, it corresponds to a subspace
/// that must contain a fixed point. Essentially, there must be a fixed point state
/// that agrees with the observation on all specified values.
pub fn colors_where_fixed_points(
    observations: Vec<Observation>,
    var_names: &[String],
    graph: &SymbolicAsyncGraph,
) -> GraphColors {
    // compute all fixed points across all colors
    let fixed_points = FixedPoints::symbolic(graph, &graph.mk_unit_colored_vertices());

    // One-by-one go over the observations and filter the color set to get colors where
    // all observations correspond to fixed points
    let mut sat_colors = fixed_points.colors();
    for observation in observations {
        // Note the observation can be a single state or a whole set corresponding to a subspace
        let observation_subspace: GraphColoredVertices =
            transform_obs_to_vertex_set(&observation, var_names, graph).unwrap();
        // We are interested in colors that exhibit fixed points corresponding to the observation,
        // which is essentially looking for colors that have non-empty intersection of states with the observation subspace
        let intersection = fixed_points.intersect(&observation_subspace);
        sat_colors = sat_colors.intersect(&intersection.colors());
    }
    sat_colors
}
