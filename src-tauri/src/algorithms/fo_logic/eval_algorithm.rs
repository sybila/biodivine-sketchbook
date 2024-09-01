use crate::algorithms::fo_logic::fol_tree::{FolTreeNode, NodeType};
use crate::algorithms::fo_logic::operator_enums::*;
use crate::algorithms::fo_logic::utils::{get_var_base_and_offset, get_var_from_implicit};
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

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
        NodeType::Binary(op, left, right) => {
            let left = eval_node(*left, graph);
            let right = eval_node(*right, graph);
            match op {
                BinaryOp::And => left.intersect(&right),
                BinaryOp::Or => left.union(&right),
                BinaryOp::Xor => eval_xor(graph, &left, &right),
                BinaryOp::Imp => eval_imp(graph, &left, &right),
                BinaryOp::Iff => eval_equiv(graph, &left, &right),
            }
        }
        NodeType::Quantifier(op, var_name, child) => match op {
            Quantifier::Exists => eval_exists(graph, &eval_node(*child, graph), &var_name),
            Quantifier::Forall => eval_forall(graph, &eval_node(*child, graph), &var_name),
        },
        NodeType::Function(fn_symbol, arguments) => {
            let name = fn_symbol.name;
            let arguments = arguments.into_iter().map(|a| *a).collect();
            if fn_symbol.is_update_fn {
                // todo - properly finish update function symbol evaluation
                eval_applied_update_function(graph, &name, arguments)
            } else {
                eval_applied_uninterpred_function(graph, &name, arguments)
            }
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
    let (base_var_name, offset) = get_var_base_and_offset(var_name).unwrap();
    let base_variable = bn.as_graph().find_variable(&base_var_name).unwrap();
    let bdd = graph
        .symbolic_context()
        .mk_extra_state_variable_is_true(base_variable, offset);
    GraphColoredVertices::new(bdd, graph.symbolic_context())
        .intersect(graph.unit_colored_vertices())
}

/// Evaluate uninterpreted function applied to given arguments.
fn eval_applied_uninterpred_function(
    graph: &SymbolicAsyncGraph,
    fn_name: &str,
    arguments: Vec<FolTreeNode>,
) -> GraphColoredVertices {
    let arguments_results: Vec<GraphColoredVertices> = arguments
        .into_iter()
        .map(|child| eval_node(child, graph))
        .collect();

    let bn = graph.as_network().unwrap();
    let function = bn.find_parameter(fn_name).unwrap();
    let fn_table = graph
        .symbolic_context()
        .get_explicit_function_table(function);

    let arguments_bdds: Vec<Bdd> = arguments_results
        .into_iter()
        .map(|x| x.into_bdd())
        .collect();

    let bdd = graph
        .symbolic_context()
        .mk_function_table_true(fn_table, &arguments_bdds);
    GraphColoredVertices::new(bdd, graph.symbolic_context())
        .intersect(graph.unit_colored_vertices())
}

/// Evaluate update function symbol applied to given arguments.
fn eval_applied_update_function(
    graph: &SymbolicAsyncGraph,
    fn_name: &str,
    arguments: Vec<FolTreeNode>,
) -> GraphColoredVertices {
    // variable whose update function this is
    let var_name = get_var_from_implicit(fn_name).unwrap();
    let bn = graph.as_network().unwrap();
    let variable = bn.as_graph().find_variable(&var_name).unwrap();

    // a) if there is update function expression, we must do some substitution,
    //    using the (modified) real expression instead of the placeholder fn symbol
    // b) if there is no update function expression, the BooleanNetwork instance
    //    internally uses a function symbol, and we can use it directly as is

    if let Some(update_fn) = bn.get_update_function(variable) {
        // convert the tree representation so that it is same as FOL tree
        let mut converted_update_fn = FolTreeNode::from_fn_update(update_fn.clone(), bn);

        // get the (automatically sorted) set of inputs of the update fn
        let input_vars = bn.regulators(variable);

        // substitute variables in the update fn's expression to actual arguments of the function
        for (input_var, expression) in input_vars.iter().zip(arguments) {
            let input_name = bn.get_variable_name(*input_var);
            converted_update_fn = converted_update_fn.substitute_variable(input_name, &expression);
        }
        eval_node(converted_update_fn, graph)
    } else {
        // we can evaluate the function normally as any other uninterpreted fn
        eval_applied_uninterpred_function(graph, fn_name, arguments)
    }
}

/// Evaluate existential quantifier.
fn eval_exists(
    graph: &SymbolicAsyncGraph,
    set: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let bn = graph.as_network().unwrap();

    // we must get the correct "extra" BDD variable from the name of the variable
    let (base_var_name, offset) = get_var_base_and_offset(var_name).unwrap();
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
