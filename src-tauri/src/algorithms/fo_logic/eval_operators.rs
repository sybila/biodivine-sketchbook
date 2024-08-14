/*
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use crate::algorithms::fo_logic::operator_enums::Term;

/// Evaluate negation respecting the allowed universe.
pub fn eval_neg(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    unit_set.minus(set)
}

/// Evaluate the implication operation.
pub fn eval_imp(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(graph, left).union(right)
}

/// Evaluate the equivalence operation.
pub fn eval_equiv(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    left.intersect(right)
        .union(&eval_neg(graph, left).intersect(&eval_neg(graph, right)))
}

/// Evaluate the non-equivalence operation (xor).
pub fn eval_xor(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(graph, &eval_equiv(graph, left, right))
}

/// Evaluate variable term.
pub fn eval_variable(
    graph: &SymbolicAsyncGraph,
    var_name: &str,
) -> GraphColoredVertices {
    todo!()
}

/// Evaluate function applied to given arguments.
pub fn eval_function(
    graph: &SymbolicAsyncGraph,
    fn_name: &str,
    arguments: Vec<(bool, Term)>
) -> GraphColoredVertices {
    todo!()
}

/// Evaluate existential quantifier.
pub fn eval_exists(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    todo!()
}

/// Evaluate universal quantifier.
pub fn eval_forall(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    todo!()
}
*/
