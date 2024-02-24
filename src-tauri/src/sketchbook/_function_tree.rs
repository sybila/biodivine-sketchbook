use crate::sketchbook::{BinaryOp, UninterpretedFnId, VarId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// todo
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FnTreeNode {
    /// A true/false FnTreeNode::Constant.
    Const(bool),
    /// References a network variable.
    Var(VarId),
    /// References a network parameter (uninterpreted function).
    /// The variable list are the arguments of the function invocation.
    UninterpretedFn(UninterpretedFnId, Vec<FnTreeNode>),
    /// Negation.
    Not(Box<FnTreeNode>),
    /// Binary boolean operation.
    Binary(BinaryOp, Box<FnTreeNode>, Box<FnTreeNode>),
}

/// Constructor and destructor utility methods. These mainly avoid unnecessary boxing
/// and exhaustive pattern matching when not necessary.
impl FnTreeNode {
    /// Create a `true` formula.
    pub fn mk_true() -> FnTreeNode {
        FnTreeNode::Const(true)
    }

    /// Create a `false` formula.
    pub fn mk_false() -> FnTreeNode {
        FnTreeNode::Const(false)
    }

    /// Create an `x` formula where `x` is a Boolean variable.
    pub fn mk_var(id: VarId) -> FnTreeNode {
        FnTreeNode::Var(id)
    }

    /// Create a `p(e_1, ..., e_k)` formula where `p` is a parameter function and `e_1` through
    /// `e_k` are general argument expressions.
    pub fn mk_param(id: UninterpretedFnId, args: &[FnTreeNode]) -> FnTreeNode {
        FnTreeNode::UninterpretedFn(id, args.to_vec())
    }

    /// Same as [Self::mk_param], but can take variable IDs as arguments directly.
    pub fn mk_basic_param(id: UninterpretedFnId, args: &[VarId]) -> FnTreeNode {
        let args = args
            .iter()
            .map(|it| FnTreeNode::mk_var(it.clone()))
            .collect::<Vec<_>>();
        FnTreeNode::UninterpretedFn(id, args)
    }

    /// Create a `!phi` formula, where `phi` is an inner `FnTreeNode`.
    pub fn mk_not(inner: FnTreeNode) -> FnTreeNode {
        FnTreeNode::Not(Box::new(inner))
    }

    /// Create a `phi 'op' psi` where `phi` and `psi` are arguments of `op` operator.
    pub fn mk_binary(op: BinaryOp, left: FnTreeNode, right: FnTreeNode) -> FnTreeNode {
        FnTreeNode::Binary(op, Box::new(left), Box::new(right))
    }

    /// Negate this function.
    pub fn negation(self) -> FnTreeNode {
        FnTreeNode::mk_not(self)
    }

    /// Create a conjunction.
    pub fn and(self, other: FnTreeNode) -> FnTreeNode {
        FnTreeNode::mk_binary(BinaryOp::And, self, other)
    }

    /// Create a disjunction.
    pub fn or(self, other: FnTreeNode) -> FnTreeNode {
        FnTreeNode::mk_binary(BinaryOp::Or, self, other)
    }

    /// Create an exclusive or.
    pub fn xor(self, other: FnTreeNode) -> FnTreeNode {
        FnTreeNode::mk_binary(BinaryOp::Xor, self, other)
    }

    /// Create an implication.
    pub fn implies(self, other: FnTreeNode) -> FnTreeNode {
        FnTreeNode::mk_binary(BinaryOp::Imp, self, other)
    }

    /// Create an equivalence.
    pub fn iff(self, other: FnTreeNode) -> FnTreeNode {
        FnTreeNode::mk_binary(BinaryOp::Iff, self, other)
    }

    /// If `Const`, return the value, otherwise return `None`.
    pub fn as_const(&self) -> Option<bool> {
        match self {
            FnTreeNode::Const(value) => Some(*value),
            _ => None,
        }
    }

    /// If `Var`, return the id, otherwise return `None`.
    pub fn as_var(&self) -> Option<VarId> {
        match self {
            FnTreeNode::Var(value) => Some(value.clone()),
            _ => None,
        }
    }

    /// If `UninterpretedFn`, return the id and args, otherwise return `None`.
    pub fn as_param(&self) -> Option<(UninterpretedFnId, &[FnTreeNode])> {
        match self {
            FnTreeNode::UninterpretedFn(id, args) => Some((id.clone(), args)),
            _ => None,
        }
    }

    /// If `Not`, return the inner function, otherwise return `None`.
    pub fn as_not(&self) -> Option<&FnTreeNode> {
        match self {
            FnTreeNode::Not(inner) => Some(inner),
            _ => None,
        }
    }

    /// If `Binary`, return the operator and left/right formulas, otherwise return `None`.
    pub fn as_binary(&self) -> Option<(&FnTreeNode, BinaryOp, &FnTreeNode)> {
        match self {
            FnTreeNode::Binary(op, l, r) => Some((l, *op, r)),
            _ => None,
        }
    }

    /// Build an expression which is equivalent to the conjunction of the given expressions.
    pub fn mk_conjunction(items: &[FnTreeNode]) -> FnTreeNode {
        if items.is_empty() {
            // Empty conjunction is `true`.
            return Self::mk_true();
        }
        if items.len() == 1 {
            return items[0].clone();
        }
        if items.len() == 2 {
            return Self::mk_binary(BinaryOp::And, items[0].clone(), items[1].clone());
        }

        let Some(first) = items.first() else {
            // Empty conjunction is `true`.
            return Self::mk_true();
        };
        let rest = Self::mk_conjunction(&items[1..]);
        first.clone().and(rest)
    }

    /// Build an expression which is equivalent to the disjunction of the given expressions.
    pub fn mk_disjunction(items: &[FnTreeNode]) -> FnTreeNode {
        if items.is_empty() {
            // Empty conjunction is `true`.
            return Self::mk_true();
        }
        if items.len() == 1 {
            return items[0].clone();
        }
        if items.len() == 2 {
            return Self::mk_binary(BinaryOp::Or, items[0].clone(), items[1].clone());
        }

        let Some(first) = items.first() else {
            // Empty conjunction is `true`.
            return Self::mk_true();
        };
        let rest = Self::mk_disjunction(&items[1..]);
        first.clone().or(rest)
    }
}

impl FnTreeNode {
    /// Try to parse an update function from a string expression.
    pub fn try_from_str(expression: &str) -> Result<FnTreeNode, String> {
        println!("{}", expression);
        todo!()
    }
}

impl Display for FnTreeNode {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
