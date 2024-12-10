use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;
use std::collections::HashMap;

// Define the maximum count of update function instances to consider (otherwise
// it may take forever to iterate the whole set, and it would also be unusable).
pub const MAX_UPDATE_FN_COUNT: usize = 1000;

/// For each variable, compute number of valid interpretations of its update
/// function present in `colors` set.
pub fn num_update_fn_variants_per_var(
    colors: &GraphColors,
    bn: &BooleanNetwork,
) -> HashMap<String, usize> {
    let graph = SymbolicAsyncGraph::new(bn).unwrap();
    let mut instance_count_map = HashMap::new();
    for var_id in graph.variables() {
        let var_name = graph.get_variable_name(var_id);
        let update_fn_projection = colors.fn_update_projection(&[var_id], &graph);
        let number_instances = update_fn_projection
            .iter()
            .take(MAX_UPDATE_FN_COUNT)
            .count();
        instance_count_map.insert(var_name.clone(), number_instances);
    }
    instance_count_map
}

/// For a given variable, get all valid interpretations of its update function present in `colors` set .
/// Variable must be present in the network.
pub fn get_update_fn_variants(
    colors: &GraphColors,
    bn: &BooleanNetwork,
    var_name: &str,
) -> Result<Vec<String>, String> {
    let graph = SymbolicAsyncGraph::new(bn).unwrap();
    let var_id = bn
        .as_graph()
        .find_variable(var_name)
        .ok_or(format!("Variable {var_name} not found"))?;
    let update_fn_projection = colors.fn_update_projection(&[var_id], &graph);

    let valid_string_updates: Vec<String> = update_fn_projection
        .iter()
        .take(MAX_UPDATE_FN_COUNT)
        .map(|valuation_singleton| valuation_singleton[0].1.to_string(bn))
        .collect();
    Ok(valid_string_updates)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::{get_update_fn_variants, num_update_fn_variants_per_var};
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

        let num_function_per_var = num_update_fn_variants_per_var(&colors, &bn);
        let expected: HashMap<String, usize> =
            HashMap::from([("a".to_string(), 6), ("b".to_string(), 3)]);
        assert_eq!(num_function_per_var, expected);

        let b_update_fns_vec = get_update_fn_variants(&colors, &bn, "b").unwrap();
        let b_update_fns = HashSet::from_iter(b_update_fns_vec.iter().cloned());
        let expected = HashSet::from(["b".to_string(), "true".to_string(), "false".to_string()]);
        assert_eq!(b_update_fns, expected);
    }
}
