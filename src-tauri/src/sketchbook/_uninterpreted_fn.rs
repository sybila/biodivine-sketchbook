use crate::sketchbook::utils::assert_name_valid;
use crate::sketchbook::{Essentiality, FnTreeNode, Monotonicity};
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
    tree: Option<FnTreeNode>,
}

impl UninterpretedFn {
    /// Create new `UninterpretedFn` object.
    pub fn new(
        name: &str,
        arity: usize,
        essentialities: Vec<Essentiality>,
        monotonicities: Vec<Monotonicity>,
        tree: Option<FnTreeNode>,
    ) -> Result<UninterpretedFn, String> {
        assert_name_valid(name)?;

        Ok(UninterpretedFn {
            name: name.to_string(),
            arity,
            essentialities,
            monotonicities,
            tree,
        })
    }

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
        self.arity = new_arity;
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
    use crate::sketchbook::UninterpretedFn;

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
}
