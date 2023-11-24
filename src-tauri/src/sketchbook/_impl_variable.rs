use crate::sketchbook::Variable;
use std::fmt::{Display, Error, Formatter};

impl Variable {
    /// Create new `Variable` objects.
    pub fn new(name_str: &str) -> Variable {
        Variable {
            name: name_str.to_string(),
        }
        // todo: perform some check on the name string?
    }

    /// Human-readable name of this variable.
    pub fn get_name(&self) -> &str {
        &self.name
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
