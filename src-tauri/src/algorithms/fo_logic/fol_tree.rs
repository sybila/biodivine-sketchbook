use crate::algorithms::fo_logic::operator_enums::*;
use crate::algorithms::fo_logic::parser::parse_fol_tokens;
use crate::algorithms::fo_logic::tokenizer::FolToken;
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate};

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Enum of possible node data types in a FOL formula syntax tree.
///
/// In particular, a node type can be:
///     - A "term" node, containing a full term (variable, constant, function applied to arguments).
///     - A "unary" node, with a `UnaryOp` and a sub-formula.
///     - A "binary" node, with a `BinaryOp` and two sub-formulae.
///     - A "quantifier" node, with a `Quantifier`, a string variable name, and a sub-formula.
///     - A "function" node,  a string variable name, and a sub-formula.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NodeType {
    Terminal(Atom),
    Unary(UnaryOp, Box<FolTreeNode>),
    Binary(BinaryOp, Box<FolTreeNode>, Box<FolTreeNode>),
    Quantifier(Quantifier, String, Box<FolTreeNode>),
    Function(FunctionSymbol, Vec<Box<FolTreeNode>>),
}

/// A single node in a syntax tree of a FOL formula.
///
/// Each node tracks its:
///     - `height`; A positive integer starting from 0 (for term nodes).
///     - `node_type`; A collection of node data represented through `NodeType`.
///     - `subform_str`; A canonical string representation of the FOL formula, which is
///     used for uniqueness testing during simplification and canonization.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FolTreeNode {
    pub formula_str: String,
    pub height: u32,
    pub node_type: NodeType,
}

impl FolTreeNode {
    /// "Parse" a new [FolTreeNode] from a list of [FolToken] objects.
    ///
    /// Note that this is a very "low-level" function. Unless you know what you are doing,
    /// you should probably use some of the functions in [crate::algorithms::fo_logic::parser] instead.
    pub fn from_tokens(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
        parse_fol_tokens(tokens)
    }

    /// Create a "quantifier" [FolTreeNode] from the given arguments.
    ///
    /// See also [NodeType::Quantifier].
    pub fn mk_quantifier(child: FolTreeNode, var: &str, op: Quantifier) -> FolTreeNode {
        FolTreeNode {
            formula_str: format!("({op} {var}: {child})"),
            height: child.height + 1,
            node_type: NodeType::Quantifier(op, var.to_string(), Box::new(child)),
        }
    }

    /// Create a "unary" [FolTreeNode] from the given arguments.
    ///
    /// See also [NodeType::Unary].
    pub fn mk_unary(child: FolTreeNode, op: UnaryOp) -> FolTreeNode {
        let subform_str = format!("({op}{child})");
        FolTreeNode {
            formula_str: subform_str,
            height: child.height + 1,
            node_type: NodeType::Unary(op, Box::new(child)),
        }
    }

    /// Create a "binary" [FolTreeNode] from the given arguments.
    ///
    /// See also [NodeType::Binary].
    pub fn mk_binary(left: FolTreeNode, right: FolTreeNode, op: BinaryOp) -> FolTreeNode {
        FolTreeNode {
            formula_str: format!("({left} {op} {right})"),
            height: cmp::max(left.height, right.height) + 1,
            node_type: NodeType::Binary(op, Box::new(left), Box::new(right)),
        }
    }

    /// Create a [FolTreeNode] representing a Boolean constant.
    ///
    /// See also [NodeType::Terminal] and [Atom::True] / [Atom::False].
    pub fn mk_constant(constant_val: bool) -> FolTreeNode {
        Self::mk_atom(Atom::from(constant_val))
    }

    /// Create a [FolTreeNode] representing a variable.
    ///
    /// See also [NodeType::Terminal] and [Atom::Var].
    pub fn mk_variable(var_name: &str) -> FolTreeNode {
        Self::mk_atom(Atom::Var(var_name.to_string()))
    }

    /// A helper function which creates a new [FolTreeNode] for the given [Atom] value.
    fn mk_atom(atom: Atom) -> FolTreeNode {
        FolTreeNode {
            formula_str: atom.to_string(),
            height: 0,
            node_type: NodeType::Terminal(atom),
        }
    }

    /// Create a [FolTreeNode] representing a function symbol applied to given arguments.
    pub fn mk_function(name: &str, inner_nodes: Vec<FolTreeNode>, is_update: bool) -> FolTreeNode {
        let max_height = inner_nodes
            .iter()
            .map(|node| node.height)
            .max()
            .unwrap_or(0);

        let child_formulas: Vec<String> = inner_nodes
            .iter()
            .map(|child| child.formula_str.clone())
            .collect();
        let args_str = child_formulas.join(", ");
        let formula_str = format!("{}({})", name, args_str);

        let inner_boxed_nodes = inner_nodes.into_iter().map(Box::new).collect();

        FolTreeNode {
            formula_str,
            height: max_height + 1,
            node_type: NodeType::Function(FunctionSymbol::new(name, is_update), inner_boxed_nodes),
        }
    }
}

impl FolTreeNode {
    /// Recursively convert `FnUpdate` expression into a `FolTreeNode`.
    /// `FnUpdate` is used to internally represent update functions. All these expressions are
    /// also valid FOL expressions, and we need to work with both at once sometimes.
    ///
    /// The provided BN gives context for variable and parameter IDs.
    pub fn from_fn_update(fn_update: FnUpdate, bn_context: &BooleanNetwork) -> FolTreeNode {
        match fn_update {
            FnUpdate::Const(value) => FolTreeNode::mk_constant(value),
            FnUpdate::Var(id) => {
                // in BN, the var's ID is a number and its name is a string we use for variables in formulas
                let var_id_str = bn_context.get_variable_name(id);
                FolTreeNode::mk_variable(var_id_str)
            }
            FnUpdate::Not(inner) => {
                let inner_transformed = Self::from_fn_update(*inner, bn_context);
                FolTreeNode::mk_unary(inner_transformed, UnaryOp::Not)
            }
            FnUpdate::Binary(op, l, r) => {
                let binary_converted = BinaryOp::from(op);
                let l_transformed = Self::from_fn_update(*l, bn_context);
                let r_transformed = Self::from_fn_update(*r, bn_context);
                FolTreeNode::mk_binary(l_transformed, r_transformed, binary_converted)
            }
            FnUpdate::Param(id, args) => {
                let fn_id_str = bn_context[id].get_name();

                let args_transformed: Vec<FolTreeNode> = args
                    .into_iter()
                    .map(|f| Self::from_fn_update(f, bn_context))
                    .collect();
                FolTreeNode::mk_function(fn_id_str, args_transformed, false)
            }
        }
    }

    /// Create a copy of this [FolTreeNode] with every occurrence of variable `var` substituted
    /// for [FolTreeNode] `expression`. It is up to you to ensure the variable is free and not
    /// quantified (otherwise weird things may happen).
    ///
    /// Note that free variables are not supported in standard FOL formulas. This function is
    /// intended for a special case when we are modifying update function expressions before we
    /// propagate them into FOL properties. For instance, having update function `f_A = A & B` and
    /// expression `\forall x, y: f_A(x, g(y))`, we need to do substitutions `A` -> `x` and `B` -> `g(y)`
    /// so that we can receive the expression `\forall x, y: (x & g(y))`.
    pub fn substitute_free_variable(&self, var: &str, expression: &FolTreeNode) -> FolTreeNode {
        match &self.node_type {
            // rename vars in terminal state-var nodes
            NodeType::Terminal(ref atom) => match atom {
                Atom::Var(name) => {
                    if name == var {
                        expression.clone()
                    } else {
                        self.clone()
                    }
                }
                // constants are always automatically fine
                _ => self.clone(),
            },
            NodeType::Unary(op, child) => {
                let node = child.substitute_free_variable(var, expression);
                FolTreeNode::mk_unary(node, *op)
            }
            NodeType::Binary(op, left, right) => {
                let node1 = left.substitute_free_variable(var, expression);
                let node2 = right.substitute_free_variable(var, expression);
                FolTreeNode::mk_binary(node1, node2, *op)
            }
            NodeType::Quantifier(op, quantified_var, child) => {
                // currently do not rename variables in quantifiers, up to the user to ensure the
                // variable to be substituted is not quantified
                let node = child.substitute_free_variable(var, expression);
                FolTreeNode::mk_quantifier(node, quantified_var, *op)
            }
            // just dive one level deeper for function nodes and rename string
            NodeType::Function(fn_symbol, child_nodes) => {
                let name = fn_symbol.name.clone();
                let is_update = fn_symbol.is_update_fn;
                let new_children = child_nodes
                    .clone()
                    .into_iter()
                    .map(|node| node.substitute_free_variable(var, expression))
                    .collect();
                FolTreeNode::mk_function(&name, new_children, is_update)
            }
        }
    }

    /// Compute the set of all uniquely named FOL variables in the formula tree.
    ///
    /// Variable names are collected from the quantifiers `exists` and `forall` (which is sufficient,
    /// as the formula must not contain free variables).
    pub fn collect_quantified_fol_vars(&self) -> HashSet<String> {
        self.collect_quantified_fol_vars_recursive(HashSet::new())
    }

    fn collect_quantified_fol_vars_recursive(
        &self,
        mut seen_vars: HashSet<String>,
    ) -> HashSet<String> {
        match &self.node_type {
            NodeType::Terminal(_) => {}
            NodeType::Unary(_, child) => {
                seen_vars.extend(child.collect_quantified_fol_vars_recursive(seen_vars.clone()));
            }
            NodeType::Binary(_, left, right) => {
                seen_vars.extend(left.collect_quantified_fol_vars_recursive(seen_vars.clone()));
                seen_vars.extend(right.collect_quantified_fol_vars_recursive(seen_vars.clone()));
            }
            // collect variables from quantifier nodes (bind, exists, forall)
            NodeType::Quantifier(_, var_name, child) => {
                seen_vars.insert(var_name.clone()); // we do not care whether insert is successful
                seen_vars.extend(child.collect_quantified_fol_vars_recursive(seen_vars.clone()));
            }
            NodeType::Function(_, child_nodes) => {
                for child in child_nodes {
                    seen_vars
                        .extend(child.collect_quantified_fol_vars_recursive(seen_vars.clone()));
                }
            }
        }
        seen_vars
    }

    /// Compute the set of all unique function symbols (with arities) in the formula tree.
    ///
    /// If some function symbol is used with more than one arity, return error.
    pub fn collect_unique_fn_symbols(&self) -> Result<HashMap<String, usize>, String> {
        let mut seen_symbols = HashMap::new();
        self.collect_unique_fn_symbols_recursive(&mut seen_symbols)?;
        Ok(seen_symbols)
    }

    fn collect_unique_fn_symbols_recursive(
        &self,
        seen_symbols: &mut HashMap<String, usize>,
    ) -> Result<(), String> {
        match &self.node_type {
            NodeType::Terminal(_) => {}
            NodeType::Unary(_, child) => {
                child.collect_unique_fn_symbols_recursive(seen_symbols)?;
            }
            NodeType::Binary(_, left, right) => {
                left.collect_unique_fn_symbols_recursive(seen_symbols)?;
                right.collect_unique_fn_symbols_recursive(seen_symbols)?;
            }
            // collect variables from quantifier nodes (bind, exists, forall)
            NodeType::Quantifier(_, _, child) => {
                child.collect_unique_fn_symbols_recursive(seen_symbols)?;
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
                    child.collect_unique_fn_symbols_recursive(seen_symbols)?;
                }
            }
        }
        Ok(())
    }

    /// Check that all FOL variables in the formula's syntactic tree are quantified (exactly once),
    /// and rename these variables to a pseudo-canonical form that will be used in BDDs.
    /// It renames as many variables as possible to identical names, without affecting the semantics.
    ///
    /// Original instance is not modified, new updated struct is returned.
    ///
    /// The format of variable names is given by how [SymbolicContext::with_extra_state_variables]
    /// creates new extra variables. Basically, we choose a name of one BN variable (`var_base_name`),
    /// and use it as a base for extra variables `{var_base_name}_extra_{index}` (index starts at 0).
    pub fn validate_and_rename_vars(&self, var_base_name: &str) -> Result<FolTreeNode, String> {
        self.validate_and_rename_recursive(var_base_name, HashMap::new(), 0)
    }

    /// Checks that all FOL variables in the formula's syntactic tree are quantified (exactly once),
    /// and renames these variables to a pseudo-canonical form that will be used in BDDs.
    /// It renames as many variables as possible to identical names, without affecting the semantics.
    ///
    /// The format of variable names is given by how [SymbolicContext::with_extra_state_variables]
    /// creates new extra variables. Basically, we choose a name of one BN variable (`var_core_name`),
    /// and it is used as a base for extra variables `{var_base_name}_extra_{index}`.
    fn validate_and_rename_recursive(
        &self,
        var_base_name: &str,
        mut renaming_map: HashMap<String, String>,
        index: u32,
    ) -> Result<FolTreeNode, String> {
        // If we find quantifier, we add new var-name to rename_dict and increment index.
        // After we leave the quantifier's sub-formula, we remove its variable from rename_dict.
        // When we find terminal node with free variable, we rename it using the rename mapping.
        match &self.node_type {
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
                _ => Ok(self.clone()),
            },
            // just dive one level deeper for unary nodes, and rename string
            NodeType::Unary(op, child) => {
                let node =
                    child.validate_and_rename_recursive(var_base_name, renaming_map, index)?;
                Ok(FolTreeNode::mk_unary(node, *op))
            }
            // just dive deeper for binary nodes, and rename string
            NodeType::Binary(op, left, right) => {
                let node1 =
                    left.validate_and_rename_recursive(var_base_name, renaming_map.clone(), index)?;
                let node2 =
                    right.validate_and_rename_recursive(var_base_name, renaming_map, index)?;
                Ok(FolTreeNode::mk_binary(node1, node2, *op))
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
                let child_node = child.validate_and_rename_recursive(
                    var_base_name,
                    renaming_map.clone(),
                    index + 1,
                )?;

                // rename the variable in the quantifier node itself
                Ok(FolTreeNode::mk_quantifier(
                    child_node,
                    new_var_name.as_str(),
                    *op,
                ))
            }
            // just dive one level deeper for function nodes and rename string
            NodeType::Function(fn_symbol, child_nodes) => {
                let name = fn_symbol.name.clone();
                let is_update = fn_symbol.is_update_fn;
                let mut new_children = Vec::new();
                for child in child_nodes {
                    let new_child_node = child.validate_and_rename_recursive(
                        var_base_name,
                        renaming_map.clone(),
                        index,
                    )?;
                    new_children.push(new_child_node);
                }
                Ok(FolTreeNode::mk_function(&name, new_children, is_update))
            }
        }
    }
}

impl FolTreeNode {
    pub fn as_str(&self) -> &str {
        self.formula_str.as_str()
    }
}

impl fmt::Display for FolTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.formula_str)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate};

    use crate::algorithms::fo_logic::fol_tree::FolTreeNode;
    use crate::algorithms::fo_logic::operator_enums::Quantifier;
    use crate::algorithms::fo_logic::parser::parse_fol_formula;
    use crate::algorithms::fo_logic::tokenizer::try_tokenize_formula;

    #[test]
    /// Test creation, ordering, and display of FOL tree nodes.
    fn tree_generating() {
        let formula = "3 x: f(x)".to_string();

        // Test that generating trees from token lists works:
        let tokens = try_tokenize_formula(formula).unwrap();
        let formula_tree = FolTreeNode::from_tokens(&tokens).unwrap();
        let expected_tree = FolTreeNode::mk_quantifier(
            FolTreeNode::mk_function("f", vec![FolTreeNode::mk_variable("x")], false),
            "x",
            Quantifier::Exists,
        );
        assert_eq!(formula_tree, expected_tree);

        // Test display:
        let normalized_str = "(\\exists x: f(x))";
        assert_eq!(formula_tree.to_string(), normalized_str.to_string());

        // Check that display output can be parsed (note that tokens could be different due
        // to extra parentheses).
        let tokens2 = try_tokenize_formula(formula_tree.to_string()).unwrap();
        let formula_tree2 = FolTreeNode::from_tokens(&tokens2).unwrap();
        assert_eq!(formula_tree, formula_tree2);
    }

    #[test]
    /// Test conversion from FnUpdate to FolTreeNode.
    fn tree_from_fn_update() {
        // simple formula and a corresponding BN for context (with all vars and functions)
        let boolean_formula = "(f(A) => B) & false";
        let context_bn = BooleanNetwork::try_from("A-?A\nB-?B\n$A: A\n$B: f(B)").unwrap();

        let expected_fol_formula = parse_fol_formula(boolean_formula).unwrap();
        let update_fn = FnUpdate::try_from_str(boolean_formula, &context_bn).unwrap();
        let transformed_fol_formula = FolTreeNode::from_fn_update(update_fn, &context_bn);
        assert_eq!(transformed_fol_formula, expected_fol_formula);
    }

    #[test]
    /// Test substitution of variables in `FolTreeNode`.
    fn tree_substitute_variable() {
        // formula with free variables corresponding to some update fn
        let formula = parse_fol_formula("(A & B)").unwrap();

        // Substitute y with constant true
        let substitute_a = FolTreeNode::mk_constant(true);
        let substituted = formula.substitute_free_variable("A", &substitute_a);
        assert_eq!(substituted.to_string(), "(1 & B)");

        let substitute_b =
            FolTreeNode::mk_function("f", vec![FolTreeNode::mk_variable("x")], false);
        let substituted = substituted.substitute_free_variable("B", &substitute_b);
        assert_eq!(substituted.to_string(), "(1 & f(x))");
    }

    #[test]
    /// Test collecting variables from a FOL formula.
    fn tree_collect_variables() {
        // Case with three variables
        let formula =
            parse_fol_formula("(\\exists x, y: (f(x) & y)) => (\\forall z: g(z))").unwrap();
        let collected_vars = formula.collect_quantified_fol_vars();
        let expexted_vars = HashSet::from(["x".to_string(), "y".to_string(), "z".to_string()]);
        assert_eq!(collected_vars, expexted_vars);

        // Case with no variables
        let formula = parse_fol_formula("f(true, false) => g()").unwrap();
        let collected_vars = formula.collect_quantified_fol_vars();
        assert_eq!(collected_vars, HashSet::new());
    }

    #[test]
    /// Test collecting function symbols from a FOL formula.
    fn tree_collect_function_symbols() {
        // Case with two function symbols
        let formula =
            parse_fol_formula("(\\exists x, y: (f(x, y) & y)) => (\\forall z: g(z))").unwrap();
        let collected_fns = formula.collect_unique_fn_symbols().unwrap();
        let expected_fns = HashMap::from([("f".to_string(), 2), ("g".to_string(), 1)]);
        assert_eq!(collected_fns, expected_fns);

        // Case with no function symbols
        let formula = parse_fol_formula("(\\exists x, y: (x & y)) => true").unwrap();
        let collected_fns = formula.collect_unique_fn_symbols().unwrap();
        assert_eq!(collected_fns, HashMap::new());

        // Invalid case where the same function symbol is used with different arities
        let formula = parse_fol_formula("(\\exists x, y: f(x, y) => f(x))").unwrap();
        let collected_fns = formula.collect_unique_fn_symbols();
        assert!(collected_fns.is_err());
    }

    #[test]
    /// Test validating and renaming vars in a FOL formula.
    fn tree_validate_rename_vars() {
        // Valid case
        let formula = parse_fol_formula("(\\exists x, y: f(x, y)) => (\\forall z: g(z))").unwrap();
        let modified_formula = formula.validate_and_rename_vars("var").unwrap();
        let expected_formula =
            parse_fol_formula("(\\exists var_extra_0, var_extra_1: f(var_extra_0, var_extra_1)) => (\\forall var_extra_0: g(var_extra_0))").unwrap();
        assert_eq!(modified_formula, expected_formula);

        // Invalid case with doubly quantified var
        let formula = parse_fol_formula("(\\exists x, x: f(x))").unwrap();
        assert!(formula.validate_and_rename_vars("var").is_err());

        // Invalid case with unquantified var
        let formula = parse_fol_formula("f(x)").unwrap();
        assert!(formula.validate_and_rename_vars("var").is_err());
    }
}
