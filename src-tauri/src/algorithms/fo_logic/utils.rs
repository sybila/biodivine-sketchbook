use crate::algorithms::fo_logic::fol_tree::{FolTreeNode, NodeType};
use crate::algorithms::fo_logic::operator_enums::*;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use std::collections::{HashMap, HashSet};

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

/// Checks that all FOL variables in the formula's syntactic tree are quantified (exactly once),
/// and renames these variables to a pseudo-canonical form that will be used in BDDs.
/// It renames as many variables as possible to identical names, without affecting the semantics.
///
/// The format of variable names is given by how [SymbolicContext::with_extra_state_variables]
/// creates new extra variables. Basically, we choose a name of one BN variable (`var_core_name`),
/// and it is used as a base for extra variables `{var_base_name}_extra_{index}`.
pub fn validate_and_rename_vars(
    orig_tree: FolTreeNode,
    var_base_name: &str,
) -> Result<FolTreeNode, String> {
    validate_and_rename_recursive(orig_tree, var_base_name, HashMap::new(), 0)
}

/// Checks that all FOL variables in the formula's syntactic tree are quantified (exactly once),
/// and renames these variables to a pseudo-canonical form that will be used in BDDs.
/// It renames as many variables as possible to identical names, without affecting the semantics.
///
/// The format of variable names is given by how [SymbolicContext::with_extra_state_variables]
/// creates new extra variables. Basically, we choose a name of one BN variable (`var_core_name`),
/// and it is used as a base for extra variables `{var_base_name}_extra_{index}`.
fn validate_and_rename_recursive(
    orig_tree: FolTreeNode,
    var_base_name: &str,
    mut renaming_map: HashMap<String, String>,
    index: u32,
) -> Result<FolTreeNode, String> {
    // If we find quantifier, we add new var-name to rename_dict and increment index.
    // After we leave the quantifier's sub-formula, we remove its variable from rename_dict.
    // When we find terminal node with free variable, we rename it using the rename mapping.
    return match orig_tree.node_type {
        // rename vars in terminal state-var nodes
        NodeType::Terminal(ref atom) => match atom {
            Atom::Var(name) => {
                // check that variable is not free (it must be already present in mapping dict)
                if !renaming_map.contains_key(name.as_str()) {
                    return Err(format!("Variable {name} is free."));
                }
                let renamed_var = renaming_map.get(name.as_str()).unwrap();
                Ok(FolTreeNode::mk_variable(renamed_var))
            }
            // constants are always automatically fine
            _ => return Ok(orig_tree),
        },
        // just dive one level deeper for unary nodes, and rename string
        NodeType::Unary(op, child) => {
            let node = validate_and_rename_recursive(*child, var_base_name, renaming_map, index)?;
            Ok(FolTreeNode::mk_unary(node, op))
        }
        // just dive deeper for binary nodes, and rename string
        NodeType::Binary(op, left, right) => {
            let node1 =
                validate_and_rename_recursive(*left, var_base_name, renaming_map.clone(), index)?;
            let node2 = validate_and_rename_recursive(*right, var_base_name, renaming_map, index)?;
            Ok(FolTreeNode::mk_binary(node1, node2, op))
        }
        // quantifier nodes are more complicated
        NodeType::Quantifier(op, var, child) => {
            // check that var is not already quantified (we dont allow that)
            if renaming_map.contains_key(var.as_str()) {
                return Err(format!(
                    "Variable {var} is quantified several times in one sub-formula."
                ));
            }
            let new_var_name = format!("{var_base_name}_extra_{index}");
            renaming_map.insert(var.clone(), new_var_name.clone());

            // dive deeper, increment index
            let child_node = validate_and_rename_recursive(
                *child,
                var_base_name,
                renaming_map.clone(),
                index + 1,
            )?;

            // rename the variable in the quantifier node itself
            Ok(FolTreeNode::mk_quantifier(
                child_node,
                new_var_name.as_str(),
                op,
            ))
        }
        // just dive one level deeper for function nodes and rename string
        NodeType::Function(FunctionSymbol(name), child_nodes) => {
            let mut new_children = Vec::new();
            for child in child_nodes {
                let new_child_node = validate_and_rename_recursive(
                    *child,
                    var_base_name,
                    renaming_map.clone(),
                    index,
                )?;
                new_children.push(new_child_node);
            }
            Ok(FolTreeNode::mk_function(&name, new_children))
        }
    };
}

/// Check that extended symbolic graph's BDD supports enough extra variables for the evaluation of
/// the formula given by a `fol_syntactic_tree`.
pub fn check_fol_var_support(_stg: &SymbolicAsyncGraph, _fol_syntactic_tree: FolTreeNode) -> bool {
    todo!()
}
