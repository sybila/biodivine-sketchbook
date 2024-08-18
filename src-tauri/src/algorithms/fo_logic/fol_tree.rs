use crate::algorithms::fo_logic::operator_enums::*;
use crate::algorithms::fo_logic::parser::parse_fol_tokens;
use crate::algorithms::fo_logic::tokenizer::FolToken;

use std::cmp;
use std::fmt;

/// Enum of possible node data types in a FOL formula syntax tree.
///
/// In particular, a node type can be:
///     - A "term" node, containing a full term (variable, constant, function applied to arguments).
///     - A "unary" node, with a `UnaryOp` and a sub-formula.
///     - A "binary" node, with a `BinaryOp` and two sub-formulae.
///     - A "quantifier" node, with a `Quantifier`, a string variable name, and a sub-formula.
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
    /// you should probably use some of the functions in [algorithms::fo_logic::parser] instead.
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
    /// See also [NodeType::Terminal] and [Atomic::True] / [Atomic::False].
    pub fn mk_constant(constant_val: bool) -> FolTreeNode {
        Self::mk_atom(Atom::from(constant_val))
    }

    /// Create a [FolTreeNode] representing a variable.
    ///
    /// See also [NodeType::Terminal] and [Atomic::Var].
    pub fn mk_variable(var_name: &str) -> FolTreeNode {
        Self::mk_atom(Atom::Var(var_name.to_string()))
    }

    /// A helper function which creates a new [FolTreeNode] for the given [Term] value.
    fn mk_atom(atom: Atom) -> FolTreeNode {
        FolTreeNode {
            formula_str: atom.to_string(),
            height: 0,
            node_type: NodeType::Terminal(atom),
        }
    }

    /// Create a [FolTreeNode] representing a function symbol applied to given arguments.
    pub fn mk_function(name: &str, inner_nodes: Vec<FolTreeNode>) -> FolTreeNode {
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
            node_type: NodeType::Function(FunctionSymbol(name.to_string()), inner_boxed_nodes),
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
    use crate::algorithms::fo_logic::fol_tree::FolTreeNode;
    use crate::algorithms::fo_logic::tokenizer::try_tokenize_formula;

    #[test]
    /// Test creation, ordering, and display of FOL tree nodes.
    fn tree_generating() {
        let formula = "3 x: f(x)".to_string();

        // Test that generating trees from token lists works:
        let tokens = try_tokenize_formula(formula).unwrap();
        let node = FolTreeNode::from_tokens(&tokens).unwrap();

        // Test display:
        let node_str = "(\\exists x: f(x))";
        assert_eq!(node.to_string(), node_str.to_string());

        // Check that display output can be parsed (note that tokens could be different due
        // to extra parentheses).
        let tokens2 = try_tokenize_formula(node.to_string()).unwrap();
        let node2 = FolTreeNode::from_tokens(&tokens2).unwrap();
        assert_eq!(node, node2);
    }
}
