/*
//! Contains the high-level model-checking algorithm and few optimisations.

use crate::algorithms::fo_logic::eval_operators::*;
use crate::algorithms::fo_logic::fol_tree::{FolTreeNode, NodeType};
use crate::algorithms::fo_logic::operator_enums::*;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Recursively evaluate the sub-formula represented by a `node` (of a syntactic tree) on a given `graph`.
pub fn eval_node(
    node: FolTreeNode,
    graph: &SymbolicAsyncGraph,
) -> GraphColoredVertices {
    match node.node_type {
        NodeType::Terminal(atom) => match atom {
            Term::True => graph.mk_unit_colored_vertices(),
            Term::False => graph.mk_empty_colored_vertices(),
            Term::Var(name) => eval_variable(graph, name.as_str()),
            Term::Function(name, arguments) => eval_function(graph, name.as_str(), arguments),
        },
        NodeType::Unary(op, child) => match op {
            UnaryOp::Not => eval_neg(
                graph,
                &eval_node(*child, graph),
            ),
        },
        NodeType::Binary(op, left, right) => {
            match op {
                BinaryOp::And => eval_node(*left, graph)
                    .intersect(&eval_node(*right, graph)),
                BinaryOp::Or => eval_node(*left, graph)
                    .union(&eval_node(*right, graph)),
                BinaryOp::Xor => eval_xor(
                    graph,
                    &eval_node(*left, graph),
                    &eval_node(*right, graph),
                ),
                BinaryOp::Imp => eval_imp(
                    graph,
                    &eval_node(*left, graph),
                    &eval_node(*right, graph),
                ),
                BinaryOp::Iff => eval_equiv(
                    graph,
                    &eval_node(*left, graph),
                    &eval_node(*right, graph),
                ),
            }
        }
        NodeType::Quantifier(op, var_name, child) => {
            match op {
                Quantifier::Exists => eval_exists(graph, &eval_node(*child, graph), &var_name),
                Quantifier::Forall => eval_forall(graph, &eval_node(*child, graph), &var_name)
            }
        }
    }
}
 */
