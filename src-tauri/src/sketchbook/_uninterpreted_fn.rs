use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// An explicit uninterpreted fn of a `BooleanNetwork`; a function symbol with a given `name` and `arity`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct UninterpretedFn {
    name: String,
    arity: u32,
}

impl UninterpretedFn {
    /// Create new `UninterpretedFn` objects.
    pub fn new(name: &str, arity: u32) -> UninterpretedFn {
        UninterpretedFn {
            name: name.to_string(),
            arity,
        }
    }

    /// Human-readable name of this uninterpreted fn.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Read arity (number of arguments) of this uninterpreted fn.
    pub fn get_arity(&self) -> u32 {
        self.arity
    }

    /// Rename this uninterpreted fn.
    pub fn set_name(&mut self, new_name: &str) {
        // todo: perform some check on the name string - at least disallow newlines
        self.name = new_name.to_string();
    }

    /// Change arity of this uninterpreted fn.
    pub fn set_arity(&mut self, new_arity: u32) {
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
        let p = UninterpretedFn::new("f", 3);
        assert_eq!(3, p.get_arity());
        assert_eq!("f", p.get_name());
        assert_eq!("f(x_1, x_2, x_3)", p.to_string().as_str());
    }
}
