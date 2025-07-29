use crate::algorithms::fo_logic::eval_algorithm::eval_node;
use crate::algorithms::fo_logic::fol_tree::FolTreeNode;
use crate::algorithms::fo_logic::parser::parse_and_minimize_fol_formula;
use crate::algorithms::fo_logic::utils::*;
use biodivine_hctl_model_checker::postprocessing::sanitizing::sanitize_colored_vertices;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Evaluate formula given by its syntactic tree, but do not sanitize the results.
///
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring
/// FO vars or the function panics.
pub fn eval_tree_dirty(
    formula_tree: FolTreeNode,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    assert_formula_and_graph_compatible(&formula_tree, graph)?;
    Ok(eval_node(formula_tree, graph))
}

/// Evaluate formula given by its syntactic tree and sanitize the resulting set of
/// colored vertices.
///
/// The `graph` MUST support enough sets of symbolic variables to represent all
/// occurring FO vars.
pub fn eval_tree(
    formula_tree: FolTreeNode,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    // compute raw results and then sanitize the BDD
    let results = eval_tree_dirty(formula_tree, graph)?;
    Ok(sanitize_colored_vertices(graph, &results))
}

/// Parse and evaluate given formula. Return sanitized results (i.e., all extra variables
/// are removed from the BDD encoding the resulting set of colored vertices).
///
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
/// Argument `base_var_name` is for the BN var which was used as a base for extra variables.
pub fn eval_formula(
    formula: &str,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<GraphColoredVertices, String> {
    let parsed_tree = parse_and_minimize_fol_formula(formula, base_var_name)?;
    eval_tree(parsed_tree, graph)
}

/// Parse and evaluate given formula, but do not sanitize the results (leaving all extra
/// BDD variables as they are).
///
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
/// Argument `base_var_name` is for the BN var which was used as a base for extra variables.
pub fn eval_formula_dirty(
    formula: &str,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<GraphColoredVertices, String> {
    let parsed_tree = parse_and_minimize_fol_formula(formula, base_var_name)?;
    eval_tree_dirty(parsed_tree, graph)
}

/// Check if variables and function symbols contained within a given FOL formula are
/// supported in the `graph` instance (i.e., it supports the symbolic BDD variables to encode
/// it all).
fn assert_formula_and_graph_compatible(
    formula_tree: &FolTreeNode,
    graph: &SymbolicAsyncGraph,
) -> Result<(), String> {
    // check if all variables valid
    let fol_vars = formula_tree.collect_quantified_fol_vars();
    for fol_var in fol_vars {
        if !check_fol_var_support(graph, &fol_var) {
            return Err(format!(
                "Variable `{fol_var}` not found in symbolic context."
            ));
        }
    }

    // check if all functions valid
    let function_symbols = formula_tree.collect_unique_fn_symbols()?;
    for (fn_name, arity) in function_symbols.iter() {
        if let Ok(var) = get_var_from_implicit(fn_name) {
            // if this is update fn symbol, we must check the corresponding variable exists
            if !check_update_fn_support(graph.as_network().unwrap(), &var, *arity) {
                return Err(format!(
                    "Variable for update function `{fn_name}` does not exist, or its arity is different."
                ));
            }
        } else {
            // if this is uninterpreted fn symbol, we must check the corresponding parameter exists
            if !check_fn_symbol_support(graph.symbolic_context(), fn_name, *arity) {
                return Err(format!(
                    "Function `{fn_name}` with arity {arity} not found in symbolic context."
                ));
            }
        }
    }
    Ok(())
}
