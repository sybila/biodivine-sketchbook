use crate::sketchbook::FunctionTree;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Update function of a `BooleanNetwork`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UpdateFunction {
    expression: String,
    tree: FunctionTree,
}

impl UpdateFunction {
    /// Create new `UpdateFunction` from a provided expression.
    pub fn try_from_str(expression: &str) -> Result<UpdateFunction, String> {
        let syntactic_tree = FunctionTree::try_from_str(expression)?;
        Ok(UpdateFunction {
            expression: syntactic_tree.to_string(),
            tree: syntactic_tree,
        })
    }

    /// Update function's string expression.
    pub fn get_fn_expression(&self) -> &str {
        &self.expression
    }

    /// Set the update function's expression to a given string.
    pub fn set_fn_expression(&mut self, new_expression: &str) -> Result<(), String> {
        self.tree = FunctionTree::try_from_str(new_expression)?;
        self.expression = self.tree.to_string();
        Ok(())
    }
}

impl Display for UpdateFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.expression)
    }
}
