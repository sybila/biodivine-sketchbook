use crate::sketchbook::utils::assert_name_valid;
use crate::sketchbook::{
    Essentiality, FnArgument, FnTree, ModelState, Monotonicity, UninterpretedFnId, VarId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

/// An explicit uninterpreted function of a `BooleanNetwork`; a function symbol with a given `name` and `arity`.
/// Fields `essentiality_list` and `monotonicity_list` hold information regarding properties of the function
/// with respect to each of its arguments (in order).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UninterpretedFn {
    name: String,
    arguments: Vec<FnArgument>,
    tree: Option<FnTree>,
    expression: String,
}

impl UninterpretedFn {
    /// Create new `UninterpretedFn` object that has no constraints regarding monotonicity, essentiality,
    /// or the function's expression itself.
    pub fn new_without_constraints(name: &str, arity: usize) -> Result<UninterpretedFn, String> {
        assert_name_valid(name)?;

        Ok(UninterpretedFn {
            name: name.to_string(),
            arguments: vec![FnArgument::default(); arity],
            tree: None,
            expression: String::new(),
        })
    }

    /// Create uninterpreted function from another one, changing its expression.
    /// The provided original function object is consumed.
    pub fn with_new_expression(
        mut original_fn: UninterpretedFn,
        new_expression: &str,
        context: &ModelState,
        own_id: &UninterpretedFnId,
    ) -> Result<UninterpretedFn, String> {
        original_fn.set_fn_expression(new_expression, context, own_id)?;
        Ok(original_fn)
    }

    /// Create uninterpreted function from another one, substituting all occurrences of a given
    /// function symbol in the syntactic tree. The provided original function object is consumed.
    pub fn with_substituted_fn_symbol(
        mut original_fn: UninterpretedFn,
        old_id: &UninterpretedFnId,
        new_id: &UninterpretedFnId,
        context: &ModelState,
    ) -> UninterpretedFn {
        original_fn.substitute_fn_symbol(old_id, new_id, context);
        original_fn
    }

    /// Human-readable name of this uninterpreted fn.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Read arity (number of arguments) of this uninterpreted fn.
    pub fn get_arity(&self) -> usize {
        self.arguments.len()
    }

    /// Rename this uninterpreted fn.
    pub fn set_name(&mut self, new_name: &str) -> Result<(), String> {
        assert_name_valid(new_name)?;
        self.name = new_name.to_string();
        Ok(())
    }

    /// Get highest index of a variable that is actually used in the function's expression.
    /// This number might be lower than actual arity.
    fn get_highest_var_idx_in_expression(&self) -> usize {
        if let Some(tree) = &self.tree {
            let highest_idx = tree
                .collect_variables()
                .iter()
                .filter_map(|v| {
                    v.to_string()
                        .strip_prefix("var")
                        .and_then(|num_str| num_str.parse::<usize>().ok())
                })
                .max();
            if let Some(idx) = highest_idx {
                return idx;
            }
        }
        0
    }

    /// Change arity of this uninterpreted fn.
    /// If arity is made larger, default arguments (without monotonicity/essentiality constraints
    /// are added).
    /// If arity is made smaller, last arguments are dropped. These must not be used in function's
    /// expression.
    pub fn set_arity(&mut self, new_arity: usize) -> Result<(), String> {
        let arity = self.get_arity();
        if new_arity < arity {
            // if arity made smaller, check that the expression does not contain variables that
            // will be dropped
            let highest_var_idx = self.get_highest_var_idx_in_expression();
            if new_arity <= highest_var_idx {
                let msg = "Cannot change arity of a function - its expression contains variables that would become invalid.";
                return Err(msg.to_string());
            }
            self.arguments.truncate(new_arity);
        } else {
            // if arity made larger, add default arguments
            let arg_count = new_arity - arity;
            for _ in 0..arg_count {
                self.add_default_argument();
            }
        }
        Ok(())
    }

    /// Drop the last argument of the function, essentially decrementing the arity of this
    /// uninterpreted fn.
    /// The last argument must not be used in function's expression.
    pub fn drop_last_argument(&mut self) -> Result<(), String> {
        if self.get_arity() == 0 {
            return Err("Cannot drop argument of a function with zero arguments.".to_string());
        }
        self.set_arity(self.get_arity() - 1)
    }

    /// Add an argument with specified monotonicity/essentiality for this function.
    /// Argument is added at the end of the current argument list.
    pub fn add_argument(&mut self, monotonicity: Monotonicity, essentiality: Essentiality) {
        self.arguments
            .push(FnArgument::new(essentiality, monotonicity));
    }

    /// Add default argument (with unknown monotonicity/essentiality) for this function.
    /// Argument is added at the end of the current argument list.
    pub fn add_default_argument(&mut self) {
        self.arguments.push(FnArgument::default());
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
        own_id: &UninterpretedFnId,
    ) -> Result<(), String> {
        if new_expression.chars().all(|c| c.is_whitespace()) {
            self.tree = None;
            self.expression = String::new()
        } else {
            let syntactic_tree =
                FnTree::try_from_str(new_expression, context, Some((own_id, self)))?;
            self.expression = syntactic_tree.to_string(context, Some(self.get_arity()));
            self.tree = Some(syntactic_tree);
        }
        Ok(())
    }

    /// Get function's argument (`FnArgument` object) on given `index` (starting from 0).
    pub fn get_argument(&self, index: usize) -> &FnArgument {
        &self.arguments[index]
    }

    /// Get `Essentiality` of argument with given `index` (starting from 0).
    pub fn get_essential(&self, index: usize) -> &Essentiality {
        &self.arguments[index].essential
    }

    /// Get `Monotonicity` of argument with given `index` (starting from 0).
    pub fn get_monotonic(&self, index: usize) -> &Monotonicity {
        &self.arguments[index].monotonicity
    }

    /// Get list of all ordered arguments (`FnArgument` objects) of this function.
    pub fn get_all_arguments(&self) -> &Vec<FnArgument> {
        &self.arguments
    }

    /// Set properties of an argument with given `index` (starting from 0).
    pub fn set_argument(&mut self, index: usize, argument: FnArgument) -> Result<(), String> {
        if index < self.get_arity() {
            self.arguments[index] = argument;
            Ok(())
        } else {
            Err("Cannot constrain an argument on index higher than function's arity.".to_string())
        }
    }

    /// Set `Essentiality` of argument with given `index` (starting from 0).
    pub fn set_essential(&mut self, index: usize, essential: Essentiality) -> Result<(), String> {
        if index < self.get_arity() {
            self.arguments[index].essential = essential;
            Ok(())
        } else {
            Err("Cannot constrain an argument on index higher than function's arity.".to_string())
        }
    }

    /// Set `Monotonicity` of argument with given `index` (starting from 0).
    pub fn set_monotonic(&mut self, index: usize, monotone: Monotonicity) -> Result<(), String> {
        if index < self.get_arity() {
            self.arguments[index].monotonicity = monotone;
            Ok(())
        } else {
            Err("Cannot constrain an argument on index higher than function's arity.".to_string())
        }
    }

    /// Set the list of all argument properties (essentially replacing them).
    /// The number of arguments must stay the same, not changing arity.
    pub fn set_all_arguments(&mut self, argument_list: Vec<FnArgument>) -> Result<(), String> {
        if argument_list.len() == self.get_arity() {
            self.arguments = argument_list;
            Ok(())
        } else {
            Err("Provided vector has different length than arity of this function.".to_string())
        }
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

    /// Substitute all occurences of a given function symbol in the syntactic tree.
    pub fn substitute_fn_symbol(
        &mut self,
        old_id: &UninterpretedFnId,
        new_id: &UninterpretedFnId,
        context: &ModelState,
    ) {
        if let Some(tree) = &self.tree {
            let new_tree = tree.substitute_fn_symbol(old_id, new_id);
            self.expression = new_tree.to_string(context, Some(self.get_arity()));
            self.tree = Some(new_tree);
        }
    }
}

impl Display for UninterpretedFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut args = Vec::new();
        for i in 1..=self.get_arity() {
            args.push(format!("x_{}", i));
        }
        let args_str = args.join(", ");
        write!(f, "{}({})", self.name, args_str)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::{ModelState, UninterpretedFn, UninterpretedFnId};

    #[test]
    fn basic_uninterpreted_fn_test() {
        let f = UninterpretedFn::new_without_constraints("f", 3).unwrap();
        assert_eq!(3, f.get_arity());
        assert_eq!("f", f.get_name());
        assert_eq!("f(x_1, x_2, x_3)", f.to_string().as_str());
    }

    #[test]
    fn invalid_uninterpreted_fn_test() {
        let f = UninterpretedFn::new_without_constraints("f\nxyz", 3);
        assert!(f.is_err());
    }

    #[test]
    fn uninterpreted_fn_expression_test() {
        // this test is a hack, normally just edit the function's expression through the `ModelState`
        // object that owns it

        let mut context = ModelState::new();
        context.add_uninterpreted_fn_by_str("f", "f", 3).unwrap();

        let fn_id = UninterpretedFnId::new("f").unwrap();
        let mut f = UninterpretedFn::new_without_constraints("f", 3).unwrap();
        let expression = "var0 & (var1 => var2)";
        f.set_fn_expression(expression, &context, &fn_id).unwrap();
        assert_eq!(f.get_fn_expression(), expression);
    }
}
