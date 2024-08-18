use crate::algorithms::fo_logic::fol_tree::{FolTreeNode, NodeType};
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use std::collections::HashSet;

/// Compute the set of all uniquely named FOL variables in the formula tree.
///
/// Variable names are collected from the quantifiers `exists` and `forall` (which is sufficient,
/// as the formula must not contain free variables).
pub fn collect_unique_fol_vars(formula_tree: FolTreeNode) -> HashSet<String> {
    collect_unique_fol_vars_recursive(formula_tree, HashSet::new())
}

fn collect_unique_fol_vars_recursive(
    formula_tree: FolTreeNode,
    mut seen_vars: HashSet<String>,
) -> HashSet<String> {
    match formula_tree.node_type {
        NodeType::Terminal(_) => {}
        NodeType::Unary(_, child) => {
            seen_vars.extend(collect_unique_fol_vars_recursive(*child, seen_vars.clone()));
        }
        NodeType::Binary(_, left, right) => {
            seen_vars.extend(collect_unique_fol_vars_recursive(*left, seen_vars.clone()));
            seen_vars.extend(collect_unique_fol_vars_recursive(*right, seen_vars.clone()));
        }
        // collect variables from quantifier nodes (bind, exists, forall)
        NodeType::Quantifier(_, var_name, child) => {
            seen_vars.insert(var_name); // we do not care whether insert is successful
            seen_vars.extend(collect_unique_fol_vars_recursive(*child, seen_vars.clone()));
        }
        NodeType::Function(_, child_nodes) => {
            for child in child_nodes {
                seen_vars.extend(collect_unique_fol_vars_recursive(*child, seen_vars.clone()));
            }
        }
    }
    seen_vars
}

/// Check that extended symbolic graph's BDD supports enough extra variables for the evaluation of
/// the formula given by a `fol_syntactic_tree`.
pub fn check_fol_var_support(_stg: &SymbolicAsyncGraph, _fol_syntactic_tree: FolTreeNode) -> bool {
    todo!()
}
