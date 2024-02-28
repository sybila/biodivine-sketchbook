use crate::sketchbook::{BinaryOp, ModelState, UninterpretedFnId, VarId};
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
    /// References a "fake" network variable that corresponds to an argument of an uninterpreted fn.
    FakeVar(VarId),
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
        fake_vars: Option<usize>,
    ) -> Result<FnTree, String> {
        println!("{}", expression);
        let bn_context = if let Some(n) = fake_vars {
            model.to_fake_bn_with_params(n)
        } else {
            model.to_empty_bn_with_params()
        };
        let fn_update = FnUpdate::try_from_str(expression, &bn_context)?;
        let fn_tree = Self::from_fn_update(fn_update, model, fake_vars)?;
        Ok(fn_tree)
    }

    /// Convert this update function to a string, taking IDs from the provided `ModelState`.
    pub fn to_string(
        &self,
        model: &ModelState,
        fake_vars: Option<usize>,
    ) -> Result<String, String> {
        let fn_update = self.to_fn_update(model, fake_vars)?;
        let bn_context = if let Some(n) = fake_vars {
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
        fake_vars: Option<usize>,
    ) -> Result<FnTree, String> {
        let bn_context = if let Some(n) = fake_vars {
            model.to_fake_bn_with_params(n)
        } else {
            model.to_empty_bn_with_params()
        };
        Self::from_fn_update_recursive(fn_update, model, &bn_context, fake_vars.is_some())
    }

    /// Recursively obtain the `FnTree` from a similar `FnUpdate` object of the [biodivine_lib_param_bn] library.
    /// The provided model gives context for variable and parameter IDs.
    fn from_fn_update_recursive(
        fn_update: FnUpdate,
        model: &ModelState,
        bn_context: &BooleanNetwork,
        is_fake: bool,
    ) -> Result<FnTree, String> {
        match fn_update {
            FnUpdate::Const(value) => Ok(FnTree::Const(value)),
            FnUpdate::Var(id) => {
                // in BN, the var's ID is a number and its name is a string (corresponding to variable ID here)
                let var_id_str = bn_context.get_variable_name(id);
                if is_fake {
                    let var_id = model.get_placeholder_var_id(var_id_str)?;
                    Ok(FnTree::FakeVar(var_id))
                } else {
                    let var_id = model.get_var_id(var_id_str)?;
                    Ok(FnTree::Var(var_id))
                }
            }
            FnUpdate::Not(inner) => {
                let inner_transformed =
                    Self::from_fn_update_recursive(*inner, model, bn_context, is_fake)?;
                Ok(FnTree::Not(Box::new(inner_transformed)))
            }
            FnUpdate::Binary(op, l, r) => {
                let binary_transformed = BinaryOp::from(op);
                let l_transformed = Self::from_fn_update_recursive(*l, model, bn_context, is_fake)?;
                let r_transformed = Self::from_fn_update_recursive(*r, model, bn_context, is_fake)?;
                Ok(FnTree::Binary(
                    binary_transformed,
                    Box::new(l_transformed),
                    Box::new(r_transformed),
                ))
            }
            FnUpdate::Param(id, args) => {
                let fn_id_str = bn_context[id].get_name();
                let fn_id = model.get_uninterpreted_fn_id(fn_id_str)?;

                let args_transformed: Result<Vec<FnTree>, String> = args
                    .into_iter()
                    .map(|f| Self::from_fn_update_recursive(f, model, bn_context, is_fake))
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
        fake_vars: Option<usize>,
    ) -> Result<FnUpdate, String> {
        let bn_context = if let Some(n) = fake_vars {
            model.to_fake_bn_with_params(n)
        } else {
            model.to_empty_bn_with_params()
        };
        self.to_fn_update_recursive(&bn_context, fake_vars.is_some())
    }

    /// Recursively transform the `FnTree` to a similar `FnUpdate` object of the [biodivine_lib_param_bn] library.
    /// The provided model gives context for variable and parameter IDs.
    fn to_fn_update_recursive(
        &self,
        bn_context: &BooleanNetwork,
        is_fake: bool,
    ) -> Result<FnUpdate, String> {
        match self {
            FnTree::Const(value) => Ok(FnUpdate::Const(*value)),
            FnTree::Var(var_id) if !is_fake => {
                // in BN, the var's ID is a number and its name is a string (corresponding to variable ID here)
                let bn_var_id = bn_context
                    .as_graph()
                    .find_variable(var_id.as_str())
                    .unwrap();
                Ok(FnUpdate::Var(bn_var_id))
            }
            FnTree::FakeVar(var_id) if is_fake => {
                let bn_var_id = bn_context
                    .as_graph()
                    .find_variable(var_id.as_str())
                    .unwrap();
                Ok(FnUpdate::Var(bn_var_id))
            }
            FnTree::Not(inner) => {
                let inner_transformed = inner.to_fn_update_recursive(bn_context, is_fake)?;
                Ok(FnUpdate::Not(Box::new(inner_transformed)))
            }
            FnTree::Binary(op, l, r) => {
                let binary_transformed = op.to_lib_param_bn_version();
                let l_transformed = l.to_fn_update_recursive(bn_context, is_fake)?;
                let r_transformed = r.to_fn_update_recursive(bn_context, is_fake)?;
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
                    .map(|f| f.to_fn_update_recursive(bn_context, is_fake))
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
    fn test_valid_fn() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 1).unwrap();

        let expression = "a & (b | f(b))";
        let fn_tree = FnTree::try_from_str(expression, &model, None).unwrap();
        assert_eq!(
            fn_tree.to_string(&model, None).unwrap().as_str(),
            expression
        );
    }

    #[test]
    fn test_invalid_fn() {
        let model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();

        let expression = "a & (b | f(b))";
        let fn_tree = FnTree::try_from_str(expression, &model, None);
        assert!(fn_tree.is_err());
    }
}
