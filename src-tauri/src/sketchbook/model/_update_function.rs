use crate::sketchbook::ids::{UninterpretedFnId, VarId};
use crate::sketchbook::model::{FnTree, ModelState};
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

/// Update function of a `BooleanNetwork`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
                expression: syntactic_tree.to_string(context, None),
                tree: Some(syntactic_tree),
            })
        }
    }

    /// Make an "empty" update function (same as [Self::default]).
    pub fn new_empty() -> UpdateFn {
        Self::default()
    }

    /// Get function's expression.
    pub fn get_fn_expression(&self) -> &str {
        &self.expression
    }

    /// Get function's syntax tree (or None if expression is empty).
    pub fn get_fn_tree(&self) -> &Option<FnTree> {
        &self.tree
    }

    /// Check if the update function's expression is empty (i.e., if the function
    /// is fully unspecified).
    pub fn has_empty_expression(&self) -> bool {
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
            self.expression = syntactic_tree.to_string(context, None);
            self.tree = Some(syntactic_tree);
        }
        Ok(())
    }

    /// Return a set of all variables that are actually used as inputs in this function.
    pub fn to_fn_update(&self, context: &BooleanNetwork) -> Option<FnUpdate> {
        self.tree.as_ref().map(|tree| tree.to_fn_update(context))
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

    /// Rename all occurrences of a given variable in the syntactic tree.
    pub fn change_var_id(&mut self, old_id: &VarId, new_id: &VarId, context: &ModelState) {
        if let Some(tree) = &self.tree {
            let new_tree = tree.change_var_id(old_id, new_id);
            self.expression = new_tree.to_string(context, None);
            self.tree = Some(new_tree);
        }
    }

    /// Rename all occurrences of a given function symbol in the syntactic tree.
    pub fn change_fn_id(
        &mut self,
        old_id: &UninterpretedFnId,
        new_id: &UninterpretedFnId,
        context: &ModelState,
    ) {
        if let Some(tree) = &self.tree {
            let new_tree = tree.change_fn_id(old_id, new_id);
            self.expression = new_tree.to_string(context, None);
            self.tree = Some(new_tree);
        }
    }

    /// Create update function from another one, substituting all occurrences of a given
    /// function symbol in the syntactic tree. The provided original function object is consumed.
    pub fn with_changed_fn_id(
        mut original_fn: UpdateFn,
        old_id: &UninterpretedFnId,
        new_id: &UninterpretedFnId,
        context: &ModelState,
    ) -> UpdateFn {
        original_fn.change_fn_id(old_id, new_id, context);
        original_fn
    }

    /// Create update function from another one, substituting all occurrences of a given
    /// variable in the syntactic tree. The provided original function object is consumed.
    pub fn with_changed_var_id(
        mut original_fn: UpdateFn,
        old_id: &VarId,
        new_id: &VarId,
        context: &ModelState,
    ) -> UpdateFn {
        original_fn.change_var_id(old_id, new_id, context);
        original_fn
    }
}
