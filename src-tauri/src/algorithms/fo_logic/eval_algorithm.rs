use crate::algorithms::fo_logic::fol_tree::{FolTreeNode, NodeType};
use crate::algorithms::fo_logic::operator_enums::*;

use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use regex::Regex;

/// Recursively evaluate the sub-formula represented by a `node` (of a syntactic tree) on a given `graph`.
pub fn eval_node(node: FolTreeNode, graph: &SymbolicAsyncGraph) -> GraphColoredVertices {
    match node.node_type {
        NodeType::Terminal(atom) => match atom {
            Atom::True => graph.mk_unit_colored_vertices(),
            Atom::False => graph.mk_empty_colored_vertices(),
            Atom::Var(name) => eval_variable(graph, name.as_str()),
        },
        NodeType::Unary(op, child) => match op {
            UnaryOp::Not => eval_neg(graph, &eval_node(*child, graph)),
        },
        NodeType::Binary(op, left, right) => match op {
            BinaryOp::And => eval_node(*left, graph).intersect(&eval_node(*right, graph)),
            BinaryOp::Or => eval_node(*left, graph).union(&eval_node(*right, graph)),
            BinaryOp::Xor => eval_xor(graph, &eval_node(*left, graph), &eval_node(*right, graph)),
            BinaryOp::Imp => eval_imp(graph, &eval_node(*left, graph), &eval_node(*right, graph)),
            BinaryOp::Iff => eval_equiv(graph, &eval_node(*left, graph), &eval_node(*right, graph)),
        },
        NodeType::Quantifier(op, var_name, child) => match op {
            Quantifier::Exists => eval_exists(graph, &eval_node(*child, graph), &var_name),
            Quantifier::Forall => eval_forall(graph, &eval_node(*child, graph), &var_name),
        },
        NodeType::Function(FunctionSymbol(name), arguments) => {
            let inner_results = arguments
                .into_iter()
                .map(|child| eval_node(*child, graph))
                .collect();
            eval_function(graph, &name, inner_results)
        }
    }
}

/// Evaluate negation respecting the allowed universe.
fn eval_neg(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    unit_set.minus(set)
}

/// Evaluate the implication operation.
fn eval_imp(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(graph, left).union(right)
}

/// Evaluate the equivalence operation.
fn eval_equiv(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    left.intersect(right)
        .union(&eval_neg(graph, left).intersect(&eval_neg(graph, right)))
}

/// Evaluate the non-equivalence operation (xor).
fn eval_xor(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(graph, &eval_equiv(graph, left, right))
}

/// Evaluate a variable term.
fn eval_variable(graph: &SymbolicAsyncGraph, var_name: &str) -> GraphColoredVertices {
    let bn = graph.as_network().unwrap();

    // we must get the correct "extra" BDD variable from the name of the variable
    let (base_var_name, offset) = get_base_and_offset(var_name);
    let base_variable = bn.as_graph().find_variable(&base_var_name).unwrap();
    let bdd = graph
        .symbolic_context()
        .mk_extra_state_variable_is_true(base_variable, offset);
    GraphColoredVertices::new(bdd, graph.symbolic_context())
        .intersect(graph.unit_colored_vertices())
}

/// Evaluate function applied to given arguments.
fn eval_function(
    graph: &SymbolicAsyncGraph,
    fn_name: &str,
    arguments: Vec<GraphColoredVertices>,
) -> GraphColoredVertices {
    let bn = graph.as_network().unwrap();
    let function = bn.find_parameter(fn_name).unwrap();
    let fn_table = graph
        .symbolic_context()
        .get_explicit_function_table(function);

    let arguments_bdds: Vec<Bdd> = arguments.into_iter().map(|x| x.into_bdd()).collect();

    let bdd = graph
        .symbolic_context()
        .mk_function_table_true(fn_table, &arguments_bdds);
    GraphColoredVertices::new(bdd, graph.symbolic_context())
        .intersect(graph.unit_colored_vertices())
}

/// Evaluate existential quantifier.
fn eval_exists(
    graph: &SymbolicAsyncGraph,
    set: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let bn = graph.as_network().unwrap();

    // we must get the correct "extra" BDD variable from the name of the variable
    let (base_var_name, offset) = get_base_and_offset(var_name);
    let variable = bn.as_graph().find_variable(&base_var_name).unwrap();
    let bbd_var = graph
        .symbolic_context()
        .get_extra_state_variable(variable, offset);

    let bbd_var_singleton = vec![bbd_var];
    let result_bdd = set.as_bdd().exists(&bbd_var_singleton);
    // after projection we do not need to intersect with unit bdd
    GraphColoredVertices::new(result_bdd, graph.symbolic_context())
}

/// Evaluate universal quantifier.
fn eval_forall(
    graph: &SymbolicAsyncGraph,
    set: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    eval_neg(graph, &eval_exists(graph, &eval_neg(graph, set), var_name))
}

/// For a given FOL variable name, get a base variable of the BN and offset that was used to add
/// it to the symbolic context.
fn get_base_and_offset(var_name: &str) -> (String, usize) {
    // we must get the correct "extra" BDD variable from the name of the variable
    let re = Regex::new(r"^(?P<network_variable>.+)_extra_(?P<i>\d+)$").unwrap();
    if let Some(captures) = re.captures(var_name) {
        let base_var_name = captures.name("network_variable").unwrap().as_str();
        let offset: usize = captures.name("i").unwrap().as_str().parse().unwrap();
        (base_var_name.to_string(), offset)
    } else {
        panic!("The FOL variable name string `{var_name}` did not match the expected format.");
    }
}
