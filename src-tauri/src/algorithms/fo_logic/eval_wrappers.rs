use crate::algorithms::fo_logic::eval_algorithm::eval_node;
use crate::algorithms::fo_logic::fol_tree::FolTreeNode;
use crate::algorithms::fo_logic::parser::parse_and_minimize_fol_formula;
use crate::algorithms::fo_logic::utils::*;
use biodivine_hctl_model_checker::postprocessing::sanitizing::sanitize_colored_vertices;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Evaluate each of a list of FOL formulas given by their syntax trees on a given transition `graph`.
///
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
/// Return the list of resulting sets of colored vertices (in the same order as input formulas).
///
/// This version does not sanitize the resulting BDDs ([eval_multiple_trees] does).
pub fn eval_multiple_trees_dirty(
    formula_trees: Vec<FolTreeNode>,
    graph: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // evaluate the formulas and collect results
    let mut results: Vec<GraphColoredVertices> = Vec::new();
    for parse_tree in formula_trees {
        results.push(eval_node(parse_tree, graph));
    }
    Ok(results)
}

/// Evaluate formula given by its syntactic tree, but do not sanitize the results.
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
pub fn eval_tree_dirty(
    formula_tree: FolTreeNode,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = eval_multiple_trees_dirty(vec![formula_tree], graph)?;
    Ok(result[0].clone())
}

/// Evaluate each of a list of FOL formulas given by their syntax trees on a given transition `graph`.
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
/// Return the list of resulting sets of colored vertices (in the same order as input formulas).
pub fn eval_multiple_trees(
    formula_trees: Vec<FolTreeNode>,
    graph: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // evaluate the formulas and collect results
    let results = eval_multiple_trees_dirty(formula_trees, graph)?;

    // sanitize the results' bdds - get rid of additional bdd vars used for HCTL vars
    let sanitized_results: Vec<GraphColoredVertices> = results
        .iter()
        .map(|x| sanitize_colored_vertices(graph, x))
        .collect();
    Ok(sanitized_results)
}

/// Evaluate formula given by its syntactic tree.
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
/// Return the resulting set of colored vertices.
pub fn eval_tree(
    formula_tree: FolTreeNode,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = eval_multiple_trees(vec![formula_tree], graph)?;
    Ok(result[0].clone())
}

/// Parse given FOL formulas list into syntactic trees and perform compatibility check with
/// the provided `graph` (i.e., check if `graph` object supports all needed symbolic variables).
///
/// Argument `base_var_name` is for the BN var which was used as a base for extra variables.
fn parse_and_validate(
    formulas: Vec<&str>,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<Vec<FolTreeNode>, String> {
    // parse all the formulas and check that graph supports enough FOL vars and contains correct functions
    let mut parsed_trees = Vec::new();
    for formula in formulas {
        let tree = parse_and_minimize_fol_formula(formula, base_var_name)?;

        // check if all variables valid
        let fol_vars = collect_unique_fol_vars(&tree);
        for fol_var in fol_vars {
            if !check_fol_var_support(graph, &fol_var) {
                return Err(format!(
                    "Variable `{fol_var}` not found in symbolic context."
                ));
            }
        }

        // check if all functions valid
        let function_symbols = collect_unique_fn_symbols(&tree)?;
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

        parsed_trees.push(tree);
    }
    Ok(parsed_trees)
}

/// Evaluate each of a list of FOL formulas on a given transition `graph`.
/// Return the resulting sets of colored vertices (in the same order as input formulas).
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
///
/// Argument `base_var_name` is for the BN var which was used as a base for extra variables.
pub fn eval_multiple_formulas(
    formulas: Vec<&str>,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<Vec<GraphColoredVertices>, String> {
    // get the abstract syntactic trees
    let parsed_trees = parse_and_validate(formulas, graph, base_var_name)?;
    // run the main model-checking procedure on formulas trees
    eval_multiple_trees(parsed_trees, graph)
}

/// Evaluate each of a list of FOL formulas, but do not sanitize the results.
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
///
/// Argument `base_var_name` is for the BN var which was used as a base for extra variables.
pub fn eval_multiple_formulas_dirty(
    formulas: Vec<&str>,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<Vec<GraphColoredVertices>, String> {
    // get the abstract syntactic trees
    let parsed_trees = parse_and_validate(formulas, graph, base_var_name)?;
    // run the main model-checking procedure on formulas trees
    eval_multiple_trees_dirty(parsed_trees, graph)
}

/// Evaluate given formula a given transition `graph`.
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
/// Return the resulting set of colored vertices.
///
/// Argument `base_var_name` is for the BN var which was used as a base for extra variables.
pub fn eval_formula(
    formula: &str,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<GraphColoredVertices, String> {
    let result = eval_multiple_formulas(vec![formula], graph, base_var_name)?;
    Ok(result[0].clone())
}

/// Evaluate given formula, but do not sanitize the result.
/// The `graph` MUST support enough sets of symbolic variables to represent all occurring FO vars.
///
/// Argument `base_var_name` is for the BN var which was used as a base for extra variables.
pub fn eval_formula_dirty(
    formula: &str,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<GraphColoredVertices, String> {
    let result = eval_multiple_formulas_dirty(vec![formula], graph, base_var_name)?;
    Ok(result[0].clone())
}
