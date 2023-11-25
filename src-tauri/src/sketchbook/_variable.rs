use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// A type safe object for a Boolean variable of a `RegulationsState`.
///
/// Currently, it only stores the variable's `name`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Variable {
    name: String,
}

impl Variable {
    /// Create new `Variable` objects.
    pub fn new(name_str: &str) -> Variable {
        Variable {
            name: name_str.to_string(),
        }
    }

    /// Human-readable name of this variable.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Rename this variable.
    pub fn set_name(&mut self, new_name: &str) {
        // todo: perform some check on the name string - at least disallow newlines
        self.name = new_name.to_string();
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::Variable;

    #[test]
    fn test_var_creation() {
        let var_name = "variable".to_string();
        let var = Variable::new(var_name.as_str());

        assert_eq!(var.get_name(), &var_name);
        assert_eq!(var.to_string(), var_name);
    }
}
