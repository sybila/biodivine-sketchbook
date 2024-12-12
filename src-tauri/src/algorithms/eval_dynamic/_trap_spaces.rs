use crate::sketchbook::observations::{Observation, VarValue};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::trap_spaces::{NetworkColoredSpaces, SymbolicSpaceContext, TrapSpaces};
use biodivine_lib_param_bn::ExtendedBoolean;
use biodivine_lib_param_bn::Space;

/// Wrapper to compute all essential colored trap spaces (under an optional set restriction).
pub fn compute_essential_trap_spaces(
    graph: &SymbolicAsyncGraph,
    ctx: &SymbolicSpaceContext,
    restriction: Option<NetworkColoredSpaces>,
) -> NetworkColoredSpaces {
    let unit_set = if let Some(restrict_set) = restriction {
        restrict_set
    } else {
        ctx.mk_unit_colored_spaces(graph)
    };

    TrapSpaces::essential_symbolic(ctx, graph, &unit_set)
}

/// Compute colors where each given observation corresponds to essential trap spaces.
pub fn colors_where_essential_traps(
    observations: Vec<Observation>,
    var_names: &[String],
    graph: &SymbolicAsyncGraph,
    ctx: &SymbolicSpaceContext,
) -> GraphColors {
    // TODO: could be optimized by restricting the colors to subset where all observations are trap spaces

    // compute essential spaces
    let essential_traps = compute_essential_trap_spaces(graph, ctx, None);

    // one-by-one, get colors where all observation spaces are essential traps
    let mut sat_colors = essential_traps.colors();
    for observation in observations {
        let obs_space_singleton: NetworkColoredSpaces =
            encode_obs_as_singleton_space(observation, var_names, graph, ctx);
        let intersection = essential_traps.intersect(&obs_space_singleton);
        sat_colors = sat_colors.intersect(&intersection.colors());
    }
    sat_colors
}

/// Wrapper to compute all minimal colored trap spaces (under an optional set restriction).
pub fn compute_minimal_trap_spaces(
    graph: &SymbolicAsyncGraph,
    ctx: &SymbolicSpaceContext,
    restriction: Option<NetworkColoredSpaces>,
) -> NetworkColoredSpaces {
    let unit_set = if let Some(restrict_set) = restriction {
        restrict_set
    } else {
        ctx.mk_unit_colored_spaces(graph)
    };

    TrapSpaces::minimal_symbolic(ctx, graph, &unit_set)
}

/// Compute colors where each given observation corresponds to minimal trap spaces.
pub fn colors_where_minimal_traps(
    observations: Vec<Observation>,
    var_names: &[String],
    graph: &SymbolicAsyncGraph,
    ctx: &SymbolicSpaceContext,
) -> GraphColors {
    // TODO: could be optimized by restricting the colors to subset where all observations are trap spaces

    // compute minimal spaces
    let minimal_traps = compute_minimal_trap_spaces(graph, ctx, None);

    // one-by-one, get colors where all observation spaces are minimal traps
    let mut sat_colors = minimal_traps.colors();
    for observation in observations {
        let obs_space_singleton: NetworkColoredSpaces =
            encode_obs_as_singleton_space(observation, var_names, graph, ctx);
        let intersection = minimal_traps.intersect(&obs_space_singleton);
        sat_colors = sat_colors.intersect(&intersection.colors());
    }
    sat_colors
}

/// Utility to encode observation as a singleton "NetworkColoredSpaces" instance,
/// i.e., a singleton space with all the colors.
fn encode_obs_as_singleton_space(
    observation: Observation,
    var_names: &[String],
    graph: &SymbolicAsyncGraph,
    ctx: &SymbolicSpaceContext,
) -> NetworkColoredSpaces {
    let bn = graph.as_network().unwrap();

    // encode the observation into Space instance
    let mut obs_space = Space::new(bn);
    var_names.iter().enumerate().for_each(|(i, var_name)| {
        let var = bn.as_graph().find_variable(var_name).unwrap();

        match observation.get_values()[i] {
            VarValue::True => {
                obs_space[var] = ExtendedBoolean::One;
            }
            VarValue::False => {
                obs_space[var] = ExtendedBoolean::Zero;
            }
            VarValue::Any => {}
        }
    });

    // compute BDD for the space, so we can create "NetworkColoredSpaces" instance
    let obs_space_bdd = ctx.mk_space(&obs_space);
    NetworkColoredSpaces::new(obs_space_bdd, ctx)
}
