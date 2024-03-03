use crate::sketchbook::{FnTree, ModelState, UninterpretedFnId, VarId};
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

/// Update function of a `BooleanNetwork`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UpdateFn {
    expression: String,
    tree: Option<FnTree>,
}

impl Display for UpdateFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expression)
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
            Ok(UpdateFn::default())
        } else {
            let syntactic_tree = FnTree::try_from_str(expression, context, None)?;
            Ok(UpdateFn {
                expression: syntactic_tree.to_string(context, None)?,
                tree: Some(syntactic_tree),
            })
        }
    }

    /// Get function's expression.
    pub fn get_fn_expression(&self) -> &str {
        &self.expression
    }

    /// Check if the update function is empty (fully unspecified).
    pub fn is_unspecified(&self) -> bool {
        self.tree.is_none()
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

    /// Return a set of all variables that are actually used as inputs in this function.
    pub fn to_fn_update(&self, context: &BooleanNetwork) -> Option<FnUpdate> {
        self.tree
            .as_ref()
            .map(|tree| tree.to_fn_update_recursive(context))
    }

    /// Return a set of all variables that are actually used as inputs in this function.
    pub fn collect_variables(&self) -> HashSet<VarId> {
        if let Some(tree) = &self.tree {
            tree.collect_variables()
        } else {
            HashSet::new()
        }
    }

    /// Return a set of all uninterpreted fns that are actually used in this function.
    pub fn collect_fn_symbols(&self) -> HashSet<UninterpretedFnId> {
        if let Some(tree) = &self.tree {
            tree.collect_fn_symbols()
        } else {
            HashSet::new()
        }
    }

    pub fn substitute_var(&mut self, old_id: &VarId, new_id: &VarId) {
        if let Some(tree) = &self.tree {
            self.tree = Some(tree.substitute_var(old_id, new_id));
        }
    }

    pub fn substitute_fn_symbol(&mut self, old_id: &UninterpretedFnId, new_id: &UninterpretedFnId) {
        if let Some(tree) = &self.tree {
            self.tree = Some(tree.substitute_fn_symbol(old_id, new_id));
        }
    }
}
