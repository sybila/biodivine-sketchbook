use crate::sketchbook::{BinaryOp, ModelState, UninterpretedFn, UninterpretedFnId, VarId};
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate};
use serde::{Deserialize, Serialize};

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

    /// Convert this update function to a string, taking IDs from the provided `ModelState`.
    pub fn to_string(
        &self,
        model: &ModelState,
        is_uninterpreted: Option<usize>,
    ) -> Result<String, String> {
        let fn_update = self.to_fn_update(model, is_uninterpreted)?;
        let bn_context = if let Some(n) = is_uninterpreted {
            model.to_fake_bn_with_params(n)
        } else {
            model.to_empty_bn_with_params()
        };
        Ok(fn_update.to_string(&bn_context))
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
    /// The provided model gives context for variable and parameter IDs.
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

    /// Transform the `FnTree` to a similar `FnUpdate` object of the [biodivine_lib_param_bn] library.
    /// The provided model gives context for variable and parameter IDs.
    fn to_fn_update(
        &self,
        model: &ModelState,
        is_uninterpreted: Option<usize>,
    ) -> Result<FnUpdate, String> {
        // TODO: get rid of a `model` argument and just make the BN out of tree variables
        let bn_context = if let Some(n) = is_uninterpreted {
            model.to_fake_bn_with_params(n)
        } else {
            model.to_empty_bn_with_params()
        };
        self.to_fn_update_recursive(&bn_context, is_uninterpreted.is_some())
    }

    /// Recursively transform the `FnTree` to a similar `FnUpdate` object of the [biodivine_lib_param_bn] library.
    /// The provided model gives context for variable and parameter IDs.
    fn to_fn_update_recursive(
        &self,
        bn_context: &BooleanNetwork,
        is_uninterpreted: bool,
    ) -> Result<FnUpdate, String> {
        match self {
            FnTree::Const(value) => Ok(FnUpdate::Const(*value)),
            FnTree::Var(var_id) if !is_uninterpreted => {
                // in BN, the var's ID is a number and its name is a string (corresponding to variable ID here)
                let bn_var_id = bn_context
                    .as_graph()
                    .find_variable(var_id.as_str())
                    .unwrap();
                Ok(FnUpdate::Var(bn_var_id))
            }
            FnTree::PlaceholderVar(var_id) if is_uninterpreted => {
                let bn_var_id = bn_context
                    .as_graph()
                    .find_variable(var_id.as_str())
                    .unwrap();
                Ok(FnUpdate::Var(bn_var_id))
            }
            FnTree::Not(inner) => {
                let inner_transformed =
                    inner.to_fn_update_recursive(bn_context, is_uninterpreted)?;
                Ok(FnUpdate::Not(Box::new(inner_transformed)))
            }
            FnTree::Binary(op, l, r) => {
                let binary_transformed = op.to_lib_param_bn_version();
                let l_transformed = l.to_fn_update_recursive(bn_context, is_uninterpreted)?;
                let r_transformed = r.to_fn_update_recursive(bn_context, is_uninterpreted)?;
                Ok(FnUpdate::Binary(
                    binary_transformed,
                    Box::new(l_transformed),
                    Box::new(r_transformed),
                ))
            }
            FnTree::UninterpretedFn(fn_id, args) => {
                let bn_param_id = bn_context.find_parameter(fn_id.as_str()).unwrap();
                let args_transformed: Result<Vec<FnUpdate>, String> = args
                    .iter()
                    .map(|f| f.to_fn_update_recursive(bn_context, is_uninterpreted))
                    .collect();
                Ok(FnUpdate::Param(bn_param_id, args_transformed?))
            }
            _ => Err("Error in a function's syntactic tree.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::{FnTree, ModelState};

    #[test]
    /// Test parsing of a valid update function's expression.
    fn test_valid_fn() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 1).unwrap();

        let expression = "a & (b | f(b))";
        let fn_tree = FnTree::try_from_str(expression, &model, None).unwrap();
        let processed_expression = fn_tree.to_string(&model, None).unwrap();
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
        let processed_expression = fn_tree.to_string(&model, Some(arity)).unwrap();
        assert_eq!(processed_expression.as_str(), expression,);
    }

    #[test]
    /// Test parsing of several invalid update functions' expressions.
    fn test_invalid_fns() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 2).unwrap();

        // try using an invalid variable
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
        model.add_uninterpreted_fn_by_str("g", "g", 1).unwrap();

        // this would be a valid update fn, but should not be a valid for an uninterpreted fn `g`
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
    }
}
