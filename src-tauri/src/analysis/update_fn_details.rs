use crate::analysis::inference_solver::FinishedInferenceSolver;
use biodivine_lib_param_bn::{
    symbolic_async_graph::{GraphColors, SymbolicAsyncGraph},
    BooleanNetwork,
};
use std::collections::{HashMap, HashSet};

// Define the maximum count of update function instances to consider (otherwise
// it may take forever to iterate the whole set, and it would also be unusable)
const MAX_COUNT: usize = 1000;

/// For each variable, compute number of valid interpretations of its update function present in the
/// satisfying `colors` (taken from the results of the solver).
pub fn num_update_fn_variants_per_var(solver: &FinishedInferenceSolver) -> HashMap<String, usize> {
    num_update_fn_variants_per_var_internal(&solver.sat_colors, &solver.bn, &solver.graph)
}

/// For each variable, compute number of valid interpretations of its update function present in `colors` set.
pub fn num_update_fn_variants_per_var_internal(
    colors: &GraphColors,
    bn: &BooleanNetwork,
    graph: &SymbolicAsyncGraph,
) -> HashMap<String, usize> {
    let mut instance_count_map = HashMap::new();
    for var_id in bn.variables() {
        let var_name = bn.get_variable_name(var_id);
        let update_fn_projection = colors.fn_update_projection(&[var_id], graph);
        let number_instances = update_fn_projection.iter().take(MAX_COUNT).count();
        instance_count_map.insert(var_name.clone(), number_instances);
    }
    instance_count_map
}

/// For a given variable, get all valid interpretations of its update function present in the
/// satisfying `colors` (taken from the results of the solver). Variable must be present in the network.
pub fn get_all_update_fn_variants(
    solver: &FinishedInferenceSolver,
    var_name: &str,
) -> Result<HashSet<String>, String> {
    get_all_update_fn_variants_internal(&solver.sat_colors, &solver.bn, &solver.graph, var_name)
}

/// For a given variable, get all valid interpretations of its update function present in `colors` set .
/// Variable must be present in the network.
pub fn get_all_update_fn_variants_internal(
    colors: &GraphColors,
    bn: &BooleanNetwork,
    graph: &SymbolicAsyncGraph,
    var_name: &str,
) -> Result<HashSet<String>, String> {
    let var_id = bn
        .as_graph()
        .find_variable(var_name)
        .ok_or(format!("Variable {var_name} not found"))?;
    let update_fn_projection = colors.fn_update_projection(&[var_id], graph);
    let valid_string_updates: HashSet<String> = update_fn_projection
        .iter()
        .take(MAX_COUNT)
        .map(|valuation_singleton| valuation_singleton[0].1.to_string(bn))
        .collect();
    Ok(valid_string_updates)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::{get_all_update_fn_variants_internal, num_update_fn_variants_per_var_internal};
    use biodivine_lib_param_bn::{symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork};

    #[test]
    fn test_projection() {
        let bn = BooleanNetwork::try_from(
            "
            a ->? a
            b ->? a
            b ->? b
            $a: f_a(a, b)
            $b: f_b(b)
        ",
        )
        .unwrap();

        let graph = SymbolicAsyncGraph::new(&bn).unwrap();
        let colors = graph.mk_unit_colors();

        let num_function_per_var = num_update_fn_variants_per_var_internal(&colors, &bn, &graph);
        let expected: HashMap<String, usize> =
            HashMap::from([("a".to_string(), 6), ("b".to_string(), 3)]);
        assert_eq!(num_function_per_var, expected);

        let b_update_fns = get_all_update_fn_variants_internal(&colors, &bn, &graph, "b").unwrap();
        let expected = HashSet::from(["b".to_string(), "true".to_string(), "false".to_string()]);
        assert_eq!(b_update_fns, expected);
    }
}
