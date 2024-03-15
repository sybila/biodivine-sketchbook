use crate::sketchbook::{BinaryOp, ModelState, UninterpretedFn, UninterpretedFnId, VarId};
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Syntactic tree of a partially defined Boolean function.
/// This might specify an update function, or a partially defined uninterpreted fn.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum FnTree {
    /// A true/false constant.
    Const(bool),
    /// References a network variable.
    Var(VarId),
    /// References a "placeholder network variable" that corresponds to an argument of an uninterpreted fn.
    PlaceholderVar(VarId),
    /// References a network parameter (uninterpreted function).
    /// The variable list are the arguments of the function invocation.
    UninterpretedFn(UninterpretedFnId, Vec<FnTree>),
    /// Negation.
    Not(Box<FnTree>),
    /// Binary Boolean operation.
    Binary(BinaryOp, Box<FnTree>, Box<FnTree>),
}

impl FnTree {
    /// Try to parse an update function from a string, taking IDs from the provided `ModelState`.
    ///
    /// `is_uninterpreted` specifies whether the expression represents an uninterpreted function,
    /// or an update function. This must be distinguished as update functions can contain network
    /// variables, but uninterpreted functions only utilize "unnamed" variables `var0`, `var1`, ...
    pub fn try_from_str(
        expression: &str,
        model: &ModelState,
        is_uninterpreted: Option<(&UninterpretedFnId, &UninterpretedFn)>,
    ) -> Result<FnTree, String> {
        let bn_context = if let Some((_, f)) = is_uninterpreted {
            model.to_fake_bn_with_params(f.get_arity())
        } else {
            model.to_empty_bn_with_params()
        };
        let fn_update = FnUpdate::try_from_str(expression, &bn_context)?;
        let fn_tree = Self::from_fn_update(fn_update, model, is_uninterpreted)?;
        Ok(fn_tree)
    }

    /// Convert this update function to a string.
    ///
    /// Currently, the transformation utilizes intermediate structs from [biodivine_lib_param_bn]
    /// library, and thus `model` is needed to provide context (regarding IDs).
    pub fn to_string(&self, model: &ModelState, is_uninterpreted: Option<usize>) -> String {
        let bn_context = if let Some(n) = is_uninterpreted {
            model.to_fake_bn_with_params(n)
        } else {
            model.to_empty_bn_with_params()
        };
        let fn_update = self.to_fn_update_recursive(&bn_context);
        fn_update.to_string(&bn_context)
    }

    /// Obtain the `FnTree` from a similar `FnUpdate` object of the [biodivine_lib_param_bn] library.
    /// The provided model gives context for variable and parameter IDs.
    fn from_fn_update(
        fn_update: FnUpdate,
        model: &ModelState,
        is_uninterpreted: Option<(&UninterpretedFnId, &UninterpretedFn)>,
    ) -> Result<FnTree, String> {
        if let Some((fn_id, f)) = is_uninterpreted {
            let bn_context = model.to_fake_bn_with_params(f.get_arity());
            Self::from_fn_update_recursive(fn_update, model, &bn_context, Some(fn_id))
        } else {
            let bn_context = model.to_empty_bn_with_params();
            Self::from_fn_update_recursive(fn_update, model, &bn_context, None)
        }
    }

    /// Recursively obtain the `FnTree` from a similar `FnUpdate` object of the [biodivine_lib_param_bn] library.
    /// The provided model and BN give context for variable and parameter IDs.
    fn from_fn_update_recursive(
        fn_update: FnUpdate,
        model: &ModelState,
        bn_context: &BooleanNetwork,
        is_uninterpreted: Option<&UninterpretedFnId>,
    ) -> Result<FnTree, String> {
        match fn_update {
            FnUpdate::Const(value) => Ok(FnTree::Const(value)),
            FnUpdate::Var(id) => {
                // in BN, the var's ID is a number and its name is a string (corresponding to variable ID here)
                let var_id_str = bn_context.get_variable_name(id);
                if is_uninterpreted.is_some() {
                    let var_id = model.get_placeholder_var_id(var_id_str)?;
                    Ok(FnTree::PlaceholderVar(var_id))
                } else {
                    let var_id = model.get_var_id(var_id_str)?;
                    Ok(FnTree::Var(var_id))
                }
            }
            FnUpdate::Not(inner) => {
                let inner_transformed =
                    Self::from_fn_update_recursive(*inner, model, bn_context, is_uninterpreted)?;
                Ok(FnTree::Not(Box::new(inner_transformed)))
            }
            FnUpdate::Binary(op, l, r) => {
                let binary_transformed = BinaryOp::from(op);
                let l_transformed =
                    Self::from_fn_update_recursive(*l, model, bn_context, is_uninterpreted)?;
                let r_transformed =
                    Self::from_fn_update_recursive(*r, model, bn_context, is_uninterpreted)?;
                Ok(FnTree::Binary(
                    binary_transformed,
                    Box::new(l_transformed),
                    Box::new(r_transformed),
                ))
            }
            FnUpdate::Param(id, args) => {
                let fn_id_str = bn_context[id].get_name();
                let fn_id = model.get_uninterpreted_fn_id(fn_id_str)?;

                // disallow recursive definition for uninterpreted fns (using a function symbol in its own expression)
                if let Some(fn_id_def) = is_uninterpreted {
                    if fn_id == *fn_id_def {
                        let msg = format!(
                            "An uninterpreted fn {fn_id} cannot be used in its own expression."
                        );
                        return Err(msg);
                    }
                }

                let args_transformed: Result<Vec<FnTree>, String> = args
                    .into_iter()
                    .map(|f| Self::from_fn_update_recursive(f, model, bn_context, is_uninterpreted))
                    .collect();
                Ok(FnTree::UninterpretedFn(fn_id, args_transformed?))
            }
        }
    }

    /// Recursively transform the `FnTree` to a similar `FnUpdate` object of the [biodivine_lib_param_bn] library.
    /// The provided BN gives context for variable and parameter IDs.
    pub(crate) fn to_fn_update_recursive(&self, bn_context: &BooleanNetwork) -> FnUpdate {
        match self {
            FnTree::Const(value) => FnUpdate::Const(*value),
            FnTree::Var(var_id) => {
                // in BN, the var's ID is a number and its name is a string (corresponding to variable ID here)
                let bn_var_id = bn_context
                    .as_graph()
                    .find_variable(var_id.as_str())
                    .unwrap();
                FnUpdate::Var(bn_var_id)
            }
            FnTree::PlaceholderVar(var_id) => {
                let bn_var_id = bn_context
                    .as_graph()
                    .find_variable(var_id.as_str())
                    .unwrap();
                FnUpdate::Var(bn_var_id)
            }
            FnTree::Not(inner) => {
                let inner_transformed = inner.to_fn_update_recursive(bn_context);
                FnUpdate::Not(Box::new(inner_transformed))
            }
            FnTree::Binary(op, l, r) => {
                let binary_transformed = op.to_lib_param_bn_version();
                let l_transformed = l.to_fn_update_recursive(bn_context);
                let r_transformed = r.to_fn_update_recursive(bn_context);
                FnUpdate::Binary(
                    binary_transformed,
                    Box::new(l_transformed),
                    Box::new(r_transformed),
                )
            }
            FnTree::UninterpretedFn(fn_id, args) => {
                let bn_param_id = bn_context.find_parameter(fn_id.as_str()).unwrap();
                let args_transformed: Vec<FnUpdate> = args
                    .iter()
                    .map(|f| f.to_fn_update_recursive(bn_context))
                    .collect();
                FnUpdate::Param(bn_param_id, args_transformed)
            }
        }
    }

    /// Return a set of all variables that are actually used in this function as arguments.
    ///
    /// Both valid `network variables` and `placeholder variables` are collected (note that
    /// these two variants can never happen to be in the same tree at the same time).
    pub fn collect_variables(&self) -> HashSet<VarId> {
        fn r_arguments(function: &FnTree, args: &mut HashSet<VarId>) {
            match function {
                FnTree::Const(_) => (),
                FnTree::Var(id) => {
                    args.insert(id.clone());
                }
                FnTree::PlaceholderVar(id) => {
                    args.insert(id.clone());
                }
                FnTree::UninterpretedFn(_, p_args) => {
                    for fun in p_args {
                        r_arguments(fun, args);
                    }
                }
                FnTree::Not(inner) => r_arguments(inner, args),
                FnTree::Binary(_, l, r) => {
                    r_arguments(l, args);
                    r_arguments(r, args);
                }
            };
        }
        let mut vars = HashSet::new();
        r_arguments(self, &mut vars);
        vars
    }

    /// Return a set of all uninterpreted functions (parameters) that are used in this function.
    pub fn collect_fn_symbols(&self) -> HashSet<UninterpretedFnId> {
        fn r_parameters(function: &FnTree, params: &mut HashSet<UninterpretedFnId>) {
            match function {
                FnTree::Const(_) => (),
                FnTree::Var(_) => (),
                FnTree::PlaceholderVar(_) => (),
                FnTree::UninterpretedFn(id, args) => {
                    params.insert(id.clone());
                    for fun in args {
                        r_parameters(fun, params);
                    }
                }
                FnTree::Not(inner) => r_parameters(inner, params),
                FnTree::Binary(_, l, r) => {
                    r_parameters(l, params);
                    r_parameters(r, params);
                }
            };
        }
        let mut params = HashSet::new();
        r_parameters(self, &mut params);
        params
    }

    /// Use this function as a template to create a new one, but substitute a given network
    /// variable's ID with a new one.
    ///
    /// This can only be used to substitute `network variables` (that appear in update functions),
    /// not placeholder variables (that appear in uninterpreted functions), since modifying the
    /// latter does not make that much sense.
    pub fn substitute_var(&self, old_id: &VarId, new_id: &VarId) -> FnTree {
        match self {
            FnTree::Const(_) => self.clone(),
            FnTree::Var(id) => {
                if id == old_id {
                    FnTree::Var(new_id.clone())
                } else {
                    self.clone()
                }
            }
            FnTree::PlaceholderVar(_) => self.clone(),
            FnTree::UninterpretedFn(id, args) => {
                let new_args = args
                    .iter()
                    .map(|it| it.substitute_var(old_id, new_id))
                    .collect::<Vec<_>>();
                FnTree::UninterpretedFn(id.clone(), new_args)
            }
            FnTree::Not(inner) => (*inner).substitute_var(old_id, new_id),
            FnTree::Binary(op, l, r) => FnTree::Binary(
                *op,
                Box::new((*l).substitute_var(old_id, new_id)),
                Box::new((*r).substitute_var(old_id, new_id)),
            ),
        }
    }

    /// Use this function as a template to create a new one, but substitute a given uninterpreted
    /// function's ID with a new one.
    pub fn substitute_fn_symbol(
        &self,
        old_id: &UninterpretedFnId,
        new_id: &UninterpretedFnId,
    ) -> FnTree {
        match self {
            FnTree::Const(_) => self.clone(),
            FnTree::Var(_) => self.clone(),
            FnTree::PlaceholderVar(_) => self.clone(),
            FnTree::UninterpretedFn(id, args) => {
                let new_args = args
                    .iter()
                    .map(|it| it.substitute_fn_symbol(old_id, new_id))
                    .collect::<Vec<_>>();
                if old_id == id {
                    FnTree::UninterpretedFn(new_id.clone(), new_args)
                } else {
                    FnTree::UninterpretedFn(id.clone(), new_args)
                }
            }
            FnTree::Not(inner) => (*inner).substitute_fn_symbol(old_id, new_id),
            FnTree::Binary(op, l, r) => FnTree::Binary(
                *op,
                Box::new((*l).substitute_fn_symbol(old_id, new_id)),
                Box::new((*r).substitute_fn_symbol(old_id, new_id)),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::{FnTree, ModelState};
    use std::collections::HashSet;

    #[test]
    /// Test parsing of a valid update function's expression.
    fn test_valid_update_fn() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 1).unwrap();

        let expression = "a & (b | f(b))";
        let fn_tree = FnTree::try_from_str(expression, &model, None).unwrap();
        let processed_expression = fn_tree.to_string(&model, None);
        assert_eq!(processed_expression.as_str(), expression);
    }

    #[test]
    /// Test parsing of a valid uninterpreted function's expression.
    fn test_valid_uninterpreted_fn() {
        let mut model = ModelState::new();
        let arity = 2;
        model.add_uninterpreted_fn_by_str("f", "f", arity).unwrap();
        model.add_uninterpreted_fn_by_str("g", "g", arity).unwrap();

        // this is a valid expression for function `g` (not for `f` though)
        let expression = "var0 & (var1 | f(var0, var0))";
        let fn_id = model.get_uninterpreted_fn_id("g").unwrap();
        let uninterpreted_fn = model.get_uninterpreted_fn(&fn_id).unwrap();
        let fn_tree =
            FnTree::try_from_str(expression, &model, Some((&fn_id, uninterpreted_fn))).unwrap();
        let processed_expression = fn_tree.to_string(&model, Some(arity));
        assert_eq!(processed_expression.as_str(), expression,);
    }

    #[test]
    /// Test parsing of several invalid update functions' expressions.
    fn test_invalid_update_fns() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 2).unwrap();

        // try using an invalid network variables
        let expression = "var0 & var1";
        let fn_tree = FnTree::try_from_str(expression, &model, None);
        assert!(fn_tree.is_err());

        // try using an invalid function symbol
        let expression = "a & (b | g(b))";
        let fn_tree = FnTree::try_from_str(expression, &model, None);
        assert!(fn_tree.is_err());

        // now try valid function symbol but with wrong arity
        let expression = "a & (b | f(b))";
        let fn_tree = FnTree::try_from_str(expression, &model, None);
        assert!(fn_tree.is_err());
    }

    #[test]
    /// Test parsing invalid uninterpreted functions' expressions.
    fn test_invalid_uninterpreted_fn() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 1).unwrap();
        model.add_uninterpreted_fn_by_str("g", "g", 2).unwrap();

        // this would be a valid `update fn`, but not an uninterpreted fn (contains network variables)
        let expression = "a & (b | f(a))";
        let fn_id = model.get_uninterpreted_fn_id("g").unwrap();
        let uninterpreted_fn = model.get_uninterpreted_fn(&fn_id).unwrap();
        let fn_tree = FnTree::try_from_str(expression, &model, Some((&fn_id, uninterpreted_fn)));
        assert!(fn_tree.is_err());

        // this is an invalid expression for the uninterpreted fn `f`, as it is recursive
        let expression = "f(var0)";
        let fn_id = model.get_uninterpreted_fn_id("f").unwrap();
        let uninterpreted_fn = model.get_uninterpreted_fn(&fn_id).unwrap();
        let fn_tree = FnTree::try_from_str(expression, &model, Some((&fn_id, uninterpreted_fn)));
        assert!(fn_tree.is_err());

        // this has higher arity (f has arity 1)
        let expression = "var0 | var1";
        let fn_id = model.get_uninterpreted_fn_id("f").unwrap();
        let uninterpreted_fn = model.get_uninterpreted_fn(&fn_id).unwrap();
        let fn_tree = FnTree::try_from_str(expression, &model, Some((&fn_id, uninterpreted_fn)));
        assert!(fn_tree.is_err());
    }

    #[test]
    /// Test variable & uninterpreted fn substitution.
    fn test_substitution() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 1).unwrap();
        model.add_uninterpreted_fn_by_str("g", "g", 1).unwrap();
        let a = model.get_var_id("a").unwrap();
        let b = model.get_var_id("b").unwrap();
        let f = model.get_uninterpreted_fn_id("f").unwrap();
        let g = model.get_uninterpreted_fn_id("g").unwrap();

        let fn_tree = FnTree::try_from_str("a & f(a)", &model, None).unwrap();

        // variable substitution
        let modified_tree = fn_tree.substitute_var(&a, &b);
        assert_eq!(modified_tree.to_string(&model, None), "b & f(b)");

        // function symbol substitution
        let modified_tree = fn_tree.substitute_fn_symbol(&f, &g);
        assert_eq!(modified_tree.to_string(&model, None), "a & g(a)");
    }

    #[test]
    /// Test collecting function symbols from function's expression.
    fn test_collect_fns() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        let fns = vec![("f", "f", 1), ("g", "g", 1), ("h", "h", 1)];
        model.add_multiple_uninterpreted_fns(fns).unwrap();
        let f = model.get_uninterpreted_fn_id("f").unwrap();
        let g = model.get_uninterpreted_fn_id("g").unwrap();

        let fn_tree = FnTree::try_from_str("a & f(a) | (g(b))", &model, None).unwrap();
        let collected_fns = fn_tree.collect_fn_symbols();
        let expected_fns = HashSet::from([f, g]);
        assert_eq!(expected_fns, collected_fns);
    }

    #[test]
    /// Test collecting variables from function's expression.
    fn test_collect_vars() {
        let variables = vec![("a", "a"), ("b", "b"), ("c", "c")];
        let mut model = ModelState::new_from_vars(variables).unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 1).unwrap();
        let a = model.get_var_id("a").unwrap();
        let b = model.get_var_id("b").unwrap();

        let fn_tree = FnTree::try_from_str("a & f(a) | (f(b))", &model, None).unwrap();
        let collected_vars = fn_tree.collect_variables();
        let expected_vars = HashSet::from([a, b]);
        assert_eq!(expected_vars, collected_vars);
    }
}
