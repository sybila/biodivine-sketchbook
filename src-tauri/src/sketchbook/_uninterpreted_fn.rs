use crate::sketchbook::utils::assert_name_valid;
use crate::sketchbook::{Essentiality, FnTree, ModelState, Monotonicity, UninterpretedFnId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// An explicit uninterpreted function of a `BooleanNetwork`; a function symbol with a given `name` and `arity`.
/// Fields `essentiality_list` and `monotonicity_list` hold information regarding properties of the function
/// with respect to each of its arguments (in order).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UninterpretedFn {
    name: String,
    arity: usize,
    essentialities: Vec<Essentiality>,
    monotonicities: Vec<Monotonicity>,
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
            arity,
            essentialities: vec![Essentiality::Unknown; arity],
            monotonicities: vec![Monotonicity::Unknown; arity],
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

    /// Human-readable name of this uninterpreted fn.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Read arity (number of arguments) of this uninterpreted fn.
    pub fn get_arity(&self) -> usize {
        self.arity
    }

    /// Rename this uninterpreted fn.
    pub fn set_name(&mut self, new_name: &str) -> Result<(), String> {
        assert_name_valid(new_name)?;
        self.name = new_name.to_string();
        Ok(())
    }

    /// Change arity of this uninterpreted fn.
    pub fn set_arity(&mut self, new_arity: usize) {
        // TODO - if arity made smaller, check that expression does not contain invalid "variables"
        self.arity = new_arity;
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
            self.expression = syntactic_tree.to_string(context, Some(self.arity))?;
            self.tree = Some(syntactic_tree);
        }
        Ok(())
    }

    /// Get `Essentiality` of argument with given `index` (starting from 0).
    pub fn get_essential(&self, index: usize) -> &Essentiality {
        &self.essentialities[index]
    }

    /// Get `Monotonicity` of argument with given `index` (starting from 0).
    pub fn get_monotonic(&self, index: usize) -> &Monotonicity {
        &self.monotonicities[index]
    }

    /// Get `Essentiality` of all arguments (in a default order).
    pub fn get_all_essential(&self) -> &Vec<Essentiality> {
        &self.essentialities
    }

    /// Get `Monotonicity` of all arguments (in a default order).
    pub fn get_all_monotonic(&self) -> &Vec<Monotonicity> {
        &self.monotonicities
    }

    /// Set `Essentiality` of argument with given `index` (starting from 0).
    pub fn set_essential(&mut self, index: usize, essential: Essentiality) -> Result<(), String> {
        if index < self.arity {
            self.essentialities[index] = essential;
            Ok(())
        } else {
            Err("Cannot constrain an argument on index higher than function's arity.".to_string())
        }
    }

    /// Set `Monotonicity` of argument with given `index` (starting from 0).
    pub fn set_monotonic(&mut self, index: usize, monotone: Monotonicity) -> Result<(), String> {
        if index < self.arity {
            self.monotonicities[index] = monotone;
            Ok(())
        } else {
            Err("Cannot constrain an argument on index higher than function's arity.".to_string())
        }
    }

    /// Set `Essentiality` of all arguments (in a default order).
    pub fn set_all_essential(
        &mut self,
        essentiality_list: Vec<Essentiality>,
    ) -> Result<(), String> {
        if essentiality_list.len() == self.arity {
            self.essentialities = essentiality_list;
            Ok(())
        } else {
            Err("Provided vector has different length than arity of this function.".to_string())
        }
    }

    /// Get `Monotonicity` of all arguments (in a default order).
    pub fn set_all_monotonic(
        &mut self,
        monotonicity_list: Vec<Monotonicity>,
    ) -> Result<(), String> {
        if monotonicity_list.len() == self.arity {
            self.monotonicities = monotonicity_list;
            Ok(())
        } else {
            Err("Provided vector has different length than arity of this function.".to_string())
        }
    }
}

impl Display for UninterpretedFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut args = Vec::new();
        for i in 1..=self.arity {
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
