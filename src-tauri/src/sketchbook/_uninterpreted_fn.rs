use crate::sketchbook::utils::assert_name_valid;
use crate::sketchbook::{Essentiality, FnTree, ModelState, Monotonicity};
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
    ) -> Result<(), String> {
        if new_expression.chars().all(|c| c.is_whitespace()) {
            self.tree = None;
            self.expression = String::new()
        } else {
            let syntactic_tree = FnTree::try_from_str(new_expression, context, Some(self.arity))?;
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
    use crate::sketchbook::{ModelState, UninterpretedFn};

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
        let mut context = ModelState::new();
        context.add_uninterpreted_fn_by_str("f", "f", 3).unwrap();

        let mut f = UninterpretedFn::new_without_constraints("f", 3).unwrap();
        let expression = "var0 & (var1 => var2)";
        f.set_fn_expression(expression, &context).unwrap();
        assert_eq!(f.get_fn_expression(), expression);
    }
}
