use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// An explicit parameter of a `BooleanNetwork`; a function symbol with a given `name` and `arity`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Parameter {
    name: String,
    arity: u32,
}

impl Parameter {
    /// Create new `Parameter` objects.
    pub fn new(name: &str, arity: u32) -> Parameter {
        Parameter {
            name: name.to_string(),
            arity,
        }
    }

    /// Human-readable name of this parameter.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Read parameter arity (number of arguments).
    pub fn get_arity(&self) -> u32 {
        self.arity
    }

    /// Rename this parameter.
    pub fn set_name(&mut self, new_name: &str) {
        // todo: perform some check on the name string - at least disallow newlines
        self.name = new_name.to_string();
    }

    /// Change arity of this parameter.
    pub fn set_arity(&mut self, new_arity: u32) {
        self.arity = new_arity;
    }
}

impl Display for Parameter {
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
    use crate::sketchbook::Parameter;

    #[test]
    fn basic_parameter_struct_test() {
        let p = Parameter::new("f", 3);
        assert_eq!(3, p.get_arity());
        assert_eq!("f", p.get_name());
        assert_eq!("f(x_1, x_2, x_3)", p.to_string().as_str());
    }
}
