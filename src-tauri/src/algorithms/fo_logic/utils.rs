use crate::algorithms::fo_logic::fol_tree::{FolTreeNode, NodeType};
use crate::algorithms::fo_logic::operator_enums::*;
use crate::sketchbook::ids::VarId;
use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, SymbolicContext};
use biodivine_lib_param_bn::BooleanNetwork;
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Compute the set of all uniquely named FOL variables in the formula tree.
///
/// Variable names are collected from the quantifiers `exists` and `forall` (which is sufficient,
/// as the formula must not contain free variables).
pub fn collect_unique_fol_vars(formula_tree: &FolTreeNode) -> HashSet<String> {
    collect_unique_fol_vars_recursive(formula_tree, HashSet::new())
}

fn collect_unique_fol_vars_recursive(
    formula_tree: &FolTreeNode,
    mut seen_vars: HashSet<String>,
) -> HashSet<String> {
    match &formula_tree.node_type {
        NodeType::Terminal(_) => {}
        NodeType::Unary(_, child) => {
            seen_vars.extend(collect_unique_fol_vars_recursive(
                child.as_ref(),
                seen_vars.clone(),
            ));
        }
        NodeType::Binary(_, left, right) => {
            seen_vars.extend(collect_unique_fol_vars_recursive(
                left.as_ref(),
                seen_vars.clone(),
            ));
            seen_vars.extend(collect_unique_fol_vars_recursive(
                right.as_ref(),
                seen_vars.clone(),
            ));
        }
        // collect variables from quantifier nodes (bind, exists, forall)
        NodeType::Quantifier(_, var_name, child) => {
            seen_vars.insert(var_name.clone()); // we do not care whether insert is successful
            seen_vars.extend(collect_unique_fol_vars_recursive(
                child.as_ref(),
                seen_vars.clone(),
            ));
        }
        NodeType::Function(_, child_nodes) => {
            for child in child_nodes {
                seen_vars.extend(collect_unique_fol_vars_recursive(
                    child.as_ref(),
                    seen_vars.clone(),
                ));
            }
        }
    }
    seen_vars
}

/// Compute the set of all unique function symbols (with arities) in the formula tree.
///
/// If some function symbol is used with more than one arity, return error.
pub fn collect_unique_fn_symbols(
    formula_tree: &FolTreeNode,
) -> Result<HashMap<String, usize>, String> {
    let mut seen_symbols = HashMap::new();
    collect_unique_fn_symbols_recursive(formula_tree, &mut seen_symbols)?;
    Ok(seen_symbols)
}

fn collect_unique_fn_symbols_recursive(
    formula_tree: &FolTreeNode,
    seen_symbols: &mut HashMap<String, usize>,
) -> Result<(), String> {
    match &formula_tree.node_type {
        NodeType::Terminal(_) => {}
        NodeType::Unary(_, child) => {
            collect_unique_fn_symbols_recursive(child.as_ref(), seen_symbols)?;
        }
        NodeType::Binary(_, left, right) => {
            collect_unique_fn_symbols_recursive(left.as_ref(), seen_symbols)?;
            collect_unique_fn_symbols_recursive(right.as_ref(), seen_symbols)?;
        }
        // collect variables from quantifier nodes (bind, exists, forall)
        NodeType::Quantifier(_, _, child) => {
            collect_unique_fn_symbols_recursive(child.as_ref(), seen_symbols)?;
        }
        NodeType::Function(fn_symbol, child_nodes) => {
            let arity = child_nodes.len();
            let name = fn_symbol.name.clone();

            if let Some(existing_arity) = seen_symbols.get(&name) {
                // if the symbol is already saved, check it has the same arity
                if *existing_arity != arity {
                    return Err(format!(
                        "Symbol {} is used with two different arities: {} and {}",
                        name, arity, existing_arity
                    ));
                }
            } else {
                // if the symbol is not saved yet, save it
                seen_symbols.insert(name, arity);
            }
            for child in child_nodes {
                collect_unique_fn_symbols_recursive(child.as_ref(), seen_symbols)?;
            }
        }
    }
    Ok(())
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
        NodeType::Function(fn_symbol, child_nodes) => {
            let name = fn_symbol.name.clone();
            let is_update = fn_symbol.is_update_fn;
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
            Ok(FolTreeNode::mk_function(&name, new_children, is_update))
        }
    };
}

/// For a given FOL variable name, get a base variable of the BN and offset that was used to add
/// it to the symbolic context.
///
/// This fn returns error if the var name format is wrong.
pub fn get_var_base_and_offset(var_name: &str) -> Result<(String, usize), String> {
    // we must get the correct "extra" BDD variable from the name of the variable
    let re = Regex::new(r"^(?P<network_variable>.+)_extra_(?P<i>\d+)$").unwrap();
    if let Some(captures) = re.captures(var_name) {
        let base_var_name = captures.name("network_variable").unwrap().as_str();
        let offset: usize = captures.name("i").unwrap().as_str().parse().unwrap();
        Ok((base_var_name.to_string(), offset))
    } else {
        Err(format!(
            "The FOL variable name string `{var_name}` did not match the expected format."
        ))
    }
}

/// If the provided function symbol corresponds to (implicit) update function for some
/// variable, get the variable's name.
/// Return Err if the symbol is not in format "f_VAR".
///
/// Note that function symbols can be either for (explicit) uninterpreted functions or
/// for (implicit) update functions. The update function symbol for variable A must be
/// in a form of "f_A".
///
/// Always expects valid `fn_symbol` name on input.
pub fn get_var_from_implicit(fn_symbol: &str) -> Result<String, String> {
    let re = Regex::new(r"^f_(?P<network_variable>.+)$").unwrap();
    if let Some(captures) = re.captures(fn_symbol) {
        let var_name = captures.name("network_variable").unwrap().as_str();
        Ok(var_name.to_string())
    } else {
        Err(format!(
            " `{fn_symbol}` is not valid symbol for an update function."
        ))
    }
}

/// Check if a given function symbol name corresponds to an (implicit) update function.
///
/// Note that function symbols can be either for (explicit) uninterpreted functions or
/// for (implicit) update functions. The update function symbol for variable A must be
/// in a form of "f_A".
///
/// Always expects valid `fn_symbol` name on input.
pub fn is_update_fn_symbol(fn_symbol: &str) -> bool {
    // this checks the format (if it is Ok it's update fn; if it is Err it's uninterpreted)
    get_var_from_implicit(fn_symbol).is_ok()
}

/// Compute a valid name for an "anonymous update function" of the corresponding variable.
///
/// todo: does not double check if there are collisions with existing params
pub fn get_implicit_function_name(variable: &VarId) -> String {
    format!("f_{}", variable.as_str())
}

/// Check that extended symbolic graph's BDD supports given extra variable.
pub fn check_fol_var_support(graph: &SymbolicAsyncGraph, var_name: &str) -> bool {
    if let Ok((base_var_name, offset)) = get_var_base_and_offset(var_name) {
        if let Some(base_var) = graph
            .as_network()
            .unwrap()
            .as_graph()
            .find_variable(&base_var_name)
        {
            let num_extra = graph
                .symbolic_context()
                .extra_state_variables(base_var)
                .len();
            return offset < num_extra;
        }
        return false;
    }
    false
}

/// Check that symbolic context supports given function symbol (parameter) of given arity.
pub fn check_fn_symbol_support(ctx: &SymbolicContext, fn_name: &str, arity: usize) -> bool {
    if let Some(param) = ctx.find_network_parameter(fn_name) {
        arity == ctx.get_network_parameter_arity(param) as usize
    } else {
        false
    }
}

/// Check that BN has given variable, and that it has given number of regulators.
pub fn check_update_fn_support(bn: &BooleanNetwork, var_name: &str, num_regulators: usize) -> bool {
    if let Some(var) = bn.as_graph().find_variable(var_name) {
        num_regulators == bn.regulators(var).len()
    } else {
        false
    }
}
