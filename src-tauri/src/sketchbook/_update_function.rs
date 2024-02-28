use crate::sketchbook::{FnTree, ModelState};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Update function of a `BooleanNetwork`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UpdateFn {
    expression: String,
    tree: Option<FnTree>,
}

impl Display for UpdateFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.expression)
    }
}

impl Default for UpdateFn {
    /// Default "empty" update function.
    fn default() -> UpdateFn {
        UpdateFn {
            expression: String::new(),
            tree: None,
        }
    }
}

impl UpdateFn {
    /// Create new `UpdateFn` from a provided expression.
    ///
    /// The expression is either a valid update fn expression or an empty (possible whitespace) string.
    pub fn try_from_str(expression: &str, context: &ModelState) -> Result<UpdateFn, String> {
        if expression.chars().all(|c| c.is_whitespace()) {
            return Ok(UpdateFn::default());
        }

        let syntactic_tree = FnTree::try_from_str(expression, context, None)?;
        Ok(UpdateFn {
            expression: syntactic_tree.to_string(context, None)?,
            tree: Some(syntactic_tree),
        })
    }

    /// Get function's expression.
    pub fn get_fn_expression(&self) -> &str {
        &self.expression
    }

    /// Set the update function's expression to a given string.
    pub fn set_fn_expression(
        &mut self,
        new_expression: &str,
        context: &ModelState,
    ) -> Result<(), String> {
        if new_expression.chars().all(|c| c.is_whitespace()) {
            self.tree = None;
            self.expression = String::new()
        } else {
            let syntactic_tree = FnTree::try_from_str(new_expression, context, None)?;
            self.expression = syntactic_tree.to_string(context, None)?;
            self.tree = Some(syntactic_tree);
        }
        Ok(())
    }
}
