use crate::sketchbook::{BinaryOp, UninterpretedFnId, VarId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// todo
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FunctionTree {
    /// A true/false constant.
    Const(bool),
    /// References a network variable.
    Var(VarId),
    /// References a network parameter (uninterpreted function).
    ///
    /// The variable list are the arguments of the function invocation.
    UninterpretedFn(UninterpretedFnId, Vec<FunctionTree>),
    /// Negation.
    Not(Box<FunctionTree>),
    /// Binary boolean operation.
    Binary(BinaryOp, Box<FunctionTree>, Box<FunctionTree>),
}

impl FunctionTree {
    /// Try to parse an update function from a string expression.
    pub fn try_from_str(expression: &str) -> Result<FunctionTree, String> {
        println!("{}", expression);
        todo!()
    }
}

impl Display for FunctionTree {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
