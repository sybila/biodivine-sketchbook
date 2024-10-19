use crate::sketchbook::utils::assert_name_valid;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// A type safe object for a Boolean variable of a `ModelState`.
///
/// Currently, it only stores the variable's `name`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Variable {
    name: String,
    annotation: String,
    // TODO: add compartments in future
}

impl Variable {
    /// Create new `Variable` instance with an annotation.
    pub fn new_annotated(name_str: &str, annotation: &str) -> Result<Variable, String> {
        assert_name_valid(name_str)?;
        Ok(Variable {
            name: name_str.to_string(),
            annotation: annotation.to_string(),
        })
    }

    /// Create new `Variable` instance. Annotation is left empty.
    pub fn new(name_str: &str) -> Result<Variable, String> {
        Self::new_annotated(name_str, "")
    }

    /// Human-readable name of this variable.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Annotation of the variable.
    pub fn get_annotation(&self) -> &str {
        &self.annotation
    }

    /// Rename this variable.
    pub fn set_name(&mut self, new_name: &str) -> Result<(), String> {
        assert_name_valid(new_name)?;
        self.name = new_name.to_string();
        Ok(())
    }

    /// Change annotation of this variable.
    pub fn set_annotation(&mut self, annotation: &str) {
        self.annotation = annotation.to_string();
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::model::Variable;

    #[test]
    fn test_variable() {
        let var_name = "v a r i a b l e 123".to_string();
        let mut var = Variable::new(var_name.as_str()).unwrap();

        assert_eq!(var.get_name(), &var_name);
        assert_eq!(var.to_string(), var_name);

        let new_name = "v a r 123".to_string();
        var.set_name(&new_name).unwrap();
        assert_eq!(var.get_name(), &new_name);
    }

    #[test]
    fn test_invalid_variable() {
        let var_name = "v\na\nr\n".to_string();
        let var = Variable::new(var_name.as_str());

        assert!(var.is_err());
    }
}
