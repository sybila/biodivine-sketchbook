use crate::sketchbook::ids::{UninterpretedFnId, VarId};
use crate::sketchbook::model::{BinaryOp, ModelState};
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Syntactic tree of a partially defined Boolean function.
/// This might specify an update function, or a partially defined uninterpreted fn.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum FnTree {
    /// A true/false constant.
    Const(bool),
    /// A network variable. The ID corresponds to some variable in the BN.
    Var(VarId),
    /// A "placeholder variable" that corresponds to a formal argument of an
    /// uninterpreted fn. This way we can build expressions for uninterpreted functions
    /// using formal arguments, independent from the actual arguments they are applied
    /// to within update functions.
    PlaceholderVar(VarId),
    /// A function symbol (also called uninterpreted function or BN parameter) applied
    /// to a list of arguments. The ID corresponds to some uninterpreted fn in the BN.
    /// The list are the arguments (parsed expressions) the function is called with.
    UninterpretedFn(UninterpretedFnId, Vec<FnTree>),
    /// Negation operation.
    Not(Box<FnTree>),
    /// Binary Boolean operation.
    Binary(BinaryOp, Box<FnTree>, Box<FnTree>),
}

/// A wrapper function for parsing update function formulas with extended error message.
/// See [FnUpdate::try_from_str] for details.
fn parse_update_fn_wrapper(
    expression: &str,
    bn_context: &BooleanNetwork,
) -> Result<FnUpdate, String> {
    let fn_update = FnUpdate::try_from_str(expression, bn_context)
        .map_err(|e| format!("Error during update function processing: {}", e))?;
    Ok(fn_update)
}

impl FnTree {
    /// Try to parse a function expression, using the IDs from the provided `ModelState`.
    ///
    /// Arg `is_uninterpreted` specifies whether the expression represents an uninterpreted
    /// function, or an update function. This must be distinguished as update functions  
    /// expressions reference network variables directly, but uninterpreted functions only
    /// utilize "unnamed" placeholder variables `var0`, `var1`, ... as their formal arguments.
    pub fn try_from_str(
        expression: &str,
        model: &ModelState,
        is_uninterpreted: Option<&UninterpretedFnId>,
    ) -> Result<FnTree, String> {
        let bn_context = if let Some(fn_id) = is_uninterpreted {
            let uninterpreted_fn = model.get_uninterpreted_fn(fn_id)?;
            model.to_fake_bn_with_params(uninterpreted_fn.get_arity())
        } else {
            model.to_empty_bn_with_params()
        };
        let fn_update = parse_update_fn_wrapper(expression, &bn_context)?;
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

    /// Obtain the `FnTree` from a similar `FnUpdate` object of the [biodivine_lib_param_bn]
    /// library. The provided model gives context for variable and parameter IDs.
    fn from_fn_update(
        fn_update: FnUpdate,
        model: &ModelState,
        is_uninterpreted: Option<&UninterpretedFnId>,
    ) -> Result<FnTree, String> {
        if let Some(fn_id) = is_uninterpreted {
            let uninterpreted_fn = model.get_uninterpreted_fn(fn_id)?;
            let bn_context = model.to_fake_bn_with_params(uninterpreted_fn.get_arity());
            Self::from_fn_update_recursive(fn_update, model, &bn_context, Some(fn_id))
        } else {
            let bn_context = model.to_empty_bn_with_params();
            Self::from_fn_update_recursive(fn_update, model, &bn_context, None)
        }
    }

    /// Recursively obtain the `FnTree` from a similar `FnUpdate` object of the [biodivine_lib_param_bn]
    /// library. The provided model and BN are used to get context for variable and parameter IDs.
    ///
    /// Argument `is_uninterpreted` is used to determine whether the result should be an uninterpreted
    /// or an update function's expression. This determines whether the variables should be translated
    /// as standard BN variables (for update fn) or placeholder variables (for uninterpreted fn).
    /// Moreover, if this should be an uninterpreted function, it must not reference its own ID
    /// to avoid recursive definitions.
    ///
    /// BN parameters are translated into uninterpreted functions.
    fn from_fn_update_recursive(
        fn_update: FnUpdate,
        model: &ModelState,
        bn_context: &BooleanNetwork,
        is_uninterpreted: Option<&UninterpretedFnId>,
    ) -> Result<FnTree, String> {
        match fn_update {
            FnUpdate::Const(value) => Ok(FnTree::Const(value)),
            FnUpdate::Var(id) => {
                // In the BooleanNetwork, variable IDs are (internal) number indices
                // More important is a variable's string name which is used in update functions
                // We thus use the BN variable name as its ID in Sketchbook
                let var_id_str = bn_context.get_variable_name(id);

                // There is a slight difference between variables in update and uninterpreted
                // functions. Update function expressions contain network variables, while uninterpreted
                // functions only contain formal arguments ("placeholder" variables).
                if let Some(fn_id) = is_uninterpreted {
                    if model.is_valid_formal_fn_argument(var_id_str, fn_id)? {
                        let var_id = VarId::new(var_id_str).unwrap(); // safe to unwrap now
                        Ok(FnTree::PlaceholderVar(var_id))
                    } else {
                        Err(format!("Variable {var_id_str} is invalid in expression of function {fn_id}. Use only function's arguments."))
                    }
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

                // disallow recursive definition for uninterpreted fns (using a function symbol in
                // its own expression)
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

    /// Recursively transform the `FnTree` to a similar `FnUpdate` object of the
    /// [biodivine_lib_param_bn] library. The provided BN gives context for variable and
    /// parameter IDs.
    ///
    /// Variables are simply translated into BN variables, and uninterpreted functions into
    /// PSBN parameters.
    pub(crate) fn to_fn_update_recursive(&self, bn_context: &BooleanNetwork) -> FnUpdate {
        match self {
            FnTree::Const(value) => FnUpdate::Const(*value),
            FnTree::Var(var_id) => {
                // In the BooleanNetwork, variable IDs are arbitrary (internal) number indices
                // More important is a variable's string name which is used in update functions
                // Sketchbook's variable ID is thus used as BN variable name
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
    /// Both valid `network variables` and `placeholder variables` are collected. Note that
    /// these two variants can never appear in the same tree, since one is used within update
    /// functions, and the other within uninterpreted functions.
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

    /// Create a new copy of this function tree, but substitute all occurances of a given
    /// network variable's ID with a new one (essentially "renaming" the variable).
    ///
    /// This method only considers `network variables` (the variables that appear in update
    /// functions). It ignores `placeholder variables` (that appear in uninterpreted functions).
    pub fn change_var_id(&self, old_id: &VarId, new_id: &VarId) -> FnTree {
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
                    .map(|it| it.change_var_id(old_id, new_id))
                    .collect::<Vec<_>>();
                FnTree::UninterpretedFn(id.clone(), new_args)
            }
            FnTree::Not(inner) => FnTree::Not(Box::new((*inner).change_var_id(old_id, new_id))),
            FnTree::Binary(op, l, r) => FnTree::Binary(
                *op,
                Box::new((*l).change_var_id(old_id, new_id)),
                Box::new((*r).change_var_id(old_id, new_id)),
            ),
        }
    }

    /// Create a new copy of this function tree, but substitute all occurances of given
    /// uninterpreted function's ID with a new one (essentially "renaming" the function).
    ///
    /// You must ensure the functions should have the same amount of arguments and this
    /// change will be consistent with the rest of the sketch. This is just a syntactic
    /// substitution.
    pub fn change_fn_id(&self, old_id: &UninterpretedFnId, new_id: &UninterpretedFnId) -> FnTree {
        match self {
            FnTree::Const(_) => self.clone(),
            FnTree::Var(_) => self.clone(),
            FnTree::PlaceholderVar(_) => self.clone(),
            FnTree::UninterpretedFn(id, args) => {
                let transformed_args = args
                    .iter()
                    .map(|it| it.change_fn_id(old_id, new_id))
                    .collect::<Vec<_>>();
                if old_id == id {
                    FnTree::UninterpretedFn(new_id.clone(), transformed_args)
                } else {
                    FnTree::UninterpretedFn(id.clone(), transformed_args)
                }
            }
            FnTree::Not(inner) => FnTree::Not(Box::new((*inner).change_fn_id(old_id, new_id))),
            FnTree::Binary(op, l, r) => FnTree::Binary(
                *op,
                Box::new((*l).change_fn_id(old_id, new_id)),
                Box::new((*r).change_fn_id(old_id, new_id)),
            ),
        }
    }

    /// Create a new copy of this function tree, but substitute all placeholder variables
    /// nodes with `FnTree` subtrees, according to a provided mapping.
    ///
    /// You must make sure the mapping covers all placeholder variables present in the expression.
    pub fn substitute_all_placeholders(
        &self,
        subst_mapping: &HashMap<VarId, FnTree>,
    ) -> Result<FnTree, String> {
        let res = match self {
            FnTree::Const(_) => self.clone(),
            FnTree::Var(_) => self.clone(),
            FnTree::PlaceholderVar(id) => {
                if let Some(new_sub_expression) = subst_mapping.get(id) {
                    new_sub_expression.clone()
                } else {
                    return Err(format!(
                        "Variable {id} is not present in the substitution mapping."
                    ));
                }
            }
            FnTree::UninterpretedFn(id, args) => {
                let new_args = args
                    .iter()
                    .map(|it| it.substitute_all_placeholders(subst_mapping))
                    .collect::<Result<Vec<_>, String>>()?;
                FnTree::UninterpretedFn(id.clone(), new_args)
            }
            FnTree::Not(inner) => FnTree::Not(Box::new(
                (*inner).substitute_all_placeholders(subst_mapping)?,
            )),
            FnTree::Binary(op, l, r) => FnTree::Binary(
                *op,
                Box::new((*l).substitute_all_placeholders(subst_mapping)?),
                Box::new((*r).substitute_all_placeholders(subst_mapping)?),
            ),
        };
        Ok(res)
    }

    /// Create a new copy of this function tree, but substitute all occurances of given
    /// uninterpreted function with its (transformed) expression.
    ///
    /// Before replacing the function symbol with its expression, the expression will be
    /// transformed. The formal parameters of the function will be substituted with the actual
    /// parameters to which the function is applied.
    ///
    /// For example, if this is tree for function `A & fn_1(B, C)`, and we have the following
    /// expression for `fn_1``: `fn_1(var1, var2) = var1 | var2`, then this will result in a
    /// tree for `A & (B | C)`.
    pub fn substitute_fn_symbol_with_expression(
        &self,
        fn_id: &UninterpretedFnId,
        fn_expression: &FnTree,
    ) -> FnTree {
        match self {
            FnTree::Const(_) => self.clone(),
            FnTree::Var(_) => self.clone(),
            FnTree::PlaceholderVar(_) => self.clone(),
            FnTree::UninterpretedFn(id, args) => {
                // recursively solve the usb-trees first
                let transformed_args = args
                    .iter()
                    .map(|it| it.substitute_fn_symbol_with_expression(fn_id, fn_expression))
                    .collect::<Vec<_>>();

                if fn_id == id {
                    let mut transformed_fn_expression = fn_expression.clone();

                    // Compute the mapping of formal -> actual function arguments (i.e., mapping from
                    // formal placeholder variables to actual expressions
                    let formal_to_actual_arg_map = transformed_args
                        .into_iter()
                        .enumerate()
                        .map(|(i, arg_expression)| {
                            (
                                VarId::new(format!("var{i}").as_str()).unwrap(),
                                arg_expression,
                            )
                        })
                        .collect::<HashMap<VarId, FnTree>>();

                    // substitute placeholder variables in uninterpreted fn expression using the mapping
                    transformed_fn_expression = transformed_fn_expression
                        .substitute_all_placeholders(&formal_to_actual_arg_map)
                        .unwrap();
                    // this transformed expression is returned instead of the original fn symbol
                    transformed_fn_expression
                } else {
                    FnTree::UninterpretedFn(id.clone(), transformed_args)
                }
            }
            FnTree::Not(inner) => FnTree::Not(Box::new(
                (*inner).substitute_fn_symbol_with_expression(fn_id, fn_expression),
            )),
            FnTree::Binary(op, l, r) => FnTree::Binary(
                *op,
                Box::new((*l).substitute_fn_symbol_with_expression(fn_id, fn_expression)),
                Box::new((*r).substitute_fn_symbol_with_expression(fn_id, fn_expression)),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::model::{FnTree, ModelState};
    use std::collections::HashSet;

    #[test]
    /// Test parsing of a valid update function's expression.
    fn test_valid_update_fn() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model
            .add_empty_uninterpreted_fn_by_str("f", "f", 1)
            .unwrap();

        let expression = "a & (b | f(b))";
        let fn_tree = FnTree::try_from_str(expression, &model, None).unwrap();
        let processed_expression = fn_tree.to_string(&model, None);
        assert_eq!(processed_expression.as_str(), expression);
    }

    #[test]
    /// Test parsing of a valid uninterpreted function's expression.
    fn test_valid_uninterpreted_fn() {
        let mut model = ModelState::new_empty();
        let arity = 2;
        model
            .add_empty_uninterpreted_fn_by_str("f", "f", arity)
            .unwrap();
        model
            .add_empty_uninterpreted_fn_by_str("g", "g", arity)
            .unwrap();

        // this is a valid expression for function `g` (not for `f` though)
        let expression = "var0 & (var1 | f(var0, var0))";
        let fn_id = model.get_uninterpreted_fn_id("g").unwrap();
        let fn_tree = FnTree::try_from_str(expression, &model, Some(&fn_id)).unwrap();
        let processed_expression = fn_tree.to_string(&model, Some(arity));
        assert_eq!(processed_expression.as_str(), expression,);
    }

    #[test]
    /// Test parsing of several invalid update functions' expressions.
    fn test_invalid_update_fns() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model
            .add_empty_uninterpreted_fn_by_str("f", "f", 2)
            .unwrap();

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
        model
            .add_empty_uninterpreted_fn_by_str("f", "f", 1)
            .unwrap();
        model
            .add_empty_uninterpreted_fn_by_str("g", "g", 2)
            .unwrap();

        // this would be a valid `update fn`, but not an uninterpreted fn (contains network variables)
        let expression = "a & (b | f(a))";
        let fn_id = model.get_uninterpreted_fn_id("g").unwrap();
        let fn_tree = FnTree::try_from_str(expression, &model, Some(&fn_id));
        assert!(fn_tree.is_err());

        // this is an invalid expression for the uninterpreted fn `f`, as it is recursive
        let expression = "f(var0)";
        let fn_id = model.get_uninterpreted_fn_id("f").unwrap();
        let fn_tree = FnTree::try_from_str(expression, &model, Some(&fn_id));
        assert!(fn_tree.is_err());

        // this has higher arity (f has arity 1)
        let expression = "var0 | var1";
        let fn_id = model.get_uninterpreted_fn_id("f").unwrap();
        let fn_tree = FnTree::try_from_str(expression, &model, Some(&fn_id));
        assert!(fn_tree.is_err());
    }

    #[test]
    /// Test variable & uninterpreted fn substitution.
    fn test_substitution() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model
            .add_empty_uninterpreted_fn_by_str("f", "f", 1)
            .unwrap();
        model
            .add_empty_uninterpreted_fn_by_str("g", "g", 1)
            .unwrap();
        let a = model.get_var_id("a").unwrap();
        let b = model.get_var_id("b").unwrap();
        let f = model.get_uninterpreted_fn_id("f").unwrap();
        let g = model.get_uninterpreted_fn_id("g").unwrap();

        let fn_tree = FnTree::try_from_str("a & f(a)", &model, None).unwrap();

        // variable substitution
        let modified_tree = fn_tree.change_var_id(&a, &b);
        assert_eq!(modified_tree.to_string(&model, None), "b & f(b)");

        // function symbol substitution
        let modified_tree = fn_tree.change_fn_id(&f, &g);
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
        model
            .add_empty_uninterpreted_fn_by_str("f", "f", 1)
            .unwrap();
        let a = model.get_var_id("a").unwrap();
        let b = model.get_var_id("b").unwrap();

        let fn_tree = FnTree::try_from_str("a & f(a) | (f(b))", &model, None).unwrap();
        let collected_vars = fn_tree.collect_variables();
        let expected_vars = HashSet::from([a, b]);
        assert_eq!(expected_vars, collected_vars);
    }
}
