use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Update function of a `BooleanNetwork`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct UpdateFunction {
    fn_string: String,
}

impl UpdateFunction {
    /// Create new `UpdateFunction` objects.
    pub fn new(fn_string: &str) -> UpdateFunction {
        UpdateFunction {
            fn_string: fn_string.to_string(),
        }
    }

    /// Update function's string expression.
    pub fn get_fn_string(&self) -> &str {
        &self.fn_string
    }

    /// Set the update function's expression via string.
    pub fn set_fn_string(&mut self, new_string: &str) {
        self.fn_string = new_string.to_string();
    }
}

impl Display for UpdateFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.fn_string)
    }
}
