use crate::sketchbook::observations::{Observation, VarValue};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::trap_spaces::{NetworkColoredSpaces, SymbolicSpaceContext};
use biodivine_lib_param_bn::ExtendedBoolean;
use biodivine_lib_param_bn::Space;

/// Transform an `observation` into a singleton "NetworkColoredSpaces" instance,
/// i.e., a singleton space with all the valid colors of the `graph`.
///
/// Variable names `var_names` correspond to the values in the observation, and
/// must be valid for the graph.
///
/// See also similar [transform_obs_to_vertex_set] or [transform_obs_to_singleton_vertex].
pub fn transform_obs_to_singleton_space(
    observation: &Observation,
    var_names: &[String],
    graph: &SymbolicAsyncGraph,
    ctx: &SymbolicSpaceContext,
) -> Result<NetworkColoredSpaces, String> {
    let bn = graph.as_network().unwrap();

    // encode the observation into Space instance
    let mut obs_space = Space::new(bn);
    var_names.iter().enumerate().try_for_each(|(i, var_name)| {
        let var_id = bn
            .as_graph()
            .find_variable(var_name)
            .ok_or(format!("Variable {var_name} is invalid."))?;

        match observation.get_values()[i] {
            VarValue::True => {
                obs_space[var_id] = ExtendedBoolean::One;
            }
            VarValue::False => {
                obs_space[var_id] = ExtendedBoolean::Zero;
            }
            VarValue::Any => {}
        }
        Ok::<(), String>(())
    })?;

    // compute BDD for the space, so we can create "NetworkColoredSpaces" instance
    let obs_space_bdd = ctx.mk_space(&obs_space);
    Ok(NetworkColoredSpaces::new(obs_space_bdd, ctx))
}

/// Get a single vertex of the STG given by the values of network variables in an `observation`.
/// The vertex is returned as `GraphColoredVertices` singleton, with all valid colours of the `graph``.
///
/// If the observation has missing values (i.e., the result is not a single vertex), Error is returned.
/// See also similar [transform_obs_to_vertex_set] or [transform_obs_to_singleton_space].
pub fn transform_obs_to_singleton_vertex(
    observation: &Observation,
    var_names: &[String],
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let vertex_set = transform_obs_to_vertex_set(observation, var_names, graph)?;
    if !vertex_set.vertices().is_singleton() {
        return Err(
            "Observation has missing values and cant be transformed into a single STG state."
                .to_string(),
        );
    }
    Ok(vertex_set)
}

/// Transform an `observation` into a corresponding set of vertices of the STG.
/// The set is returned as `GraphColoredVertices` set, with all valid colours of the `graph``.
///
/// See also similar [transform_obs_to_singleton_vertex] or [transform_obs_to_singleton_space].
pub fn transform_obs_to_vertex_set(
    observation: &Observation,
    var_names: &[String],
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let bn = graph.as_network().unwrap();
    let mut set = graph.mk_unit_colored_vertices();
    var_names.iter().enumerate().try_for_each(|(i, var_name)| {
        let var_id = bn
            .as_graph()
            .find_variable(var_name)
            .ok_or(format!("Variable {var_name} is invalid."))?;

        match observation.get_values()[i] {
            VarValue::True => {
                let var_is_true = graph.fix_network_variable(var_id, true);
                set = set.intersect(&var_is_true);
            }
            VarValue::False => {
                let var_is_false = graph.fix_network_variable(var_id, false);
                set = set.intersect(&var_is_false);
            }
            VarValue::Any => {}
        }
        Ok::<(), String>(())
    })?;

    Ok(set)
}

#[cfg(test)]
mod tests {
    use crate::algorithms::eval_dynamic::utils::{
        transform_obs_to_singleton_vertex, transform_obs_to_vertex_set,
    };
    use crate::sketchbook::observations::Observation;
    use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
    use biodivine_lib_param_bn::trap_spaces::SymbolicSpaceContext;
    use biodivine_lib_param_bn::{BooleanNetwork, Space};

    use super::transform_obs_to_singleton_space;

    #[test]
    /// Test transforming observation with all values present into coloured spaces or vertices.
    fn test_transform_fully_defined_observation() {
        // create observation over two variables A, B
        let var_names = vec!["A".to_string(), "B".to_string()];
        let obs = Observation::try_from_str("01", "o1").unwrap();

        // prepare simple BN with the same two variables and 2 parametrizations
        let bn = BooleanNetwork::try_from("A -> B\n$A:const\n$B:A").unwrap();
        let stg = SymbolicAsyncGraph::new(&bn).unwrap();
        let var_a = bn.as_graph().find_variable("A").unwrap();
        let var_b = bn.as_graph().find_variable("B").unwrap();
        let space_ctx = SymbolicSpaceContext::new(&bn);

        // check the transformed singleton spaces set
        let space_singleton =
            transform_obs_to_singleton_space(&obs, &var_names, &stg, &space_ctx).unwrap();
        assert_eq!(space_singleton.approx_cardinality(), 2.0);
        assert!(space_singleton.spaces().is_singleton());
        let inner_space = space_singleton.spaces().iter().last().unwrap();
        let expected_space = Space::from_values(&bn, vec![(var_a, false), (var_b, true)]);
        assert_eq!(inner_space, expected_space);

        // check the transformed singleton vertices set
        let vertex_singleton = transform_obs_to_singleton_vertex(&obs, &var_names, &stg).unwrap();
        assert_eq!(vertex_singleton.approx_cardinality(), 2.0);
        assert!(vertex_singleton.vertices().is_singleton());
        let inner_vertex_values = vertex_singleton.vertices().iter().last().unwrap().values();
        let expected_values = vec![false, true];
        assert_eq!(inner_vertex_values, expected_values);

        // check that all vertex/space sets match
        let vertex_set = transform_obs_to_vertex_set(&obs, &var_names, &stg).unwrap();
        assert_eq!(vertex_set, vertex_singleton);
        let space_to_vertex_set = space_singleton.to_colored_vertices(&space_ctx);
        let space_to_vertex_set_bdd = stg
            .symbolic_context()
            .transfer_from(space_to_vertex_set.as_bdd(), space_ctx.inner_context())
            .unwrap();
        assert!(space_to_vertex_set_bdd.eq(vertex_set.as_bdd()));
    }

    #[test]
    /// Test transforming observation with missing values into coloured spaces or vertices.
    fn test_transform_partially_defined_observation() {
        // create observation over two variables A, B
        let var_names = vec!["A".to_string(), "B".to_string()];
        let obs = Observation::try_from_str("0*", "o1").unwrap();

        // prepare simple BN with the same two variables and 2 parametrizations
        let bn = BooleanNetwork::try_from("A -> B\n$A:const\n$B:A").unwrap();
        let stg = SymbolicAsyncGraph::new(&bn).unwrap();
        let var_a = bn.as_graph().find_variable("A").unwrap();
        let space_ctx = SymbolicSpaceContext::new(&bn);

        // check the transformed singleton space set
        let space_singleton =
            transform_obs_to_singleton_space(&obs, &var_names, &stg, &space_ctx).unwrap();
        assert_eq!(space_singleton.approx_cardinality(), 2.0);
        assert!(space_singleton.spaces().is_singleton());
        let inner_space = space_singleton.spaces().iter().last().unwrap();
        let expected_space = Space::from_values(&bn, vec![(var_a, false)]);
        assert_eq!(inner_space, expected_space);

        // vertex singleton cant be made from partially specified obs
        let vertex_singleton = transform_obs_to_singleton_vertex(&obs, &var_names, &stg);
        assert!(vertex_singleton.is_err());

        // check the transformed vertices set
        let vertex_set = transform_obs_to_vertex_set(&obs, &var_names, &stg).unwrap();
        assert_eq!(vertex_set.approx_cardinality(), 4.0);
        assert_eq!(vertex_set.vertices().approx_cardinality(), 2.0);

        // check that vertices/space sets match
        let space_to_vertex_set = space_singleton.to_colored_vertices(&space_ctx);
        let space_to_vertex_set_bdd = stg
            .symbolic_context()
            .transfer_from(space_to_vertex_set.as_bdd(), space_ctx.inner_context())
            .unwrap();
        assert!(space_to_vertex_set_bdd.eq(vertex_set.as_bdd()));
    }
}
