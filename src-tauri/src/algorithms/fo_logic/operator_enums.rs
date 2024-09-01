use std::fmt;

// Re-export BinaryOp, since it is the only operator struct we are defining elsewhere.
// Binary operators are already defined and used in update functions, so we use it
// instead of rewriting the same code.
pub use crate::sketchbook::model::BinaryOp;

/// Enum for all possible Boolean unary operators occurring in a FOL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, Copy)]
pub enum UnaryOp {
    Not, // '!'
}

/// Enum for quantifiers in a FOL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, Copy)]
pub enum Quantifier {
    Exists, // '3' or "\exists"
    Forall, // 'V' or "\forall"
}

/// Enum for atomic terms: variables and constants.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Atom {
    Var(String), // A variable
    True,        // A true constant
    False,       // A false constant
}

/// A named function symbol with a boolean flag whether the symbol is an explicit
/// uninterpreted function, or an implicit update function implicit.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FunctionSymbol {
    pub name: String,
    pub is_update_fn: bool,
}

impl FunctionSymbol {
    pub fn new(name: &str, is_update_fn: bool) -> FunctionSymbol {
        FunctionSymbol {
            name: name.to_string(),
            is_update_fn,
        }
    }

    pub fn new_uninterpreted(name: &str) -> FunctionSymbol {
        FunctionSymbol {
            name: name.to_string(),
            is_update_fn: false,
        }
    }

    pub fn new_update(name: &str) -> FunctionSymbol {
        FunctionSymbol {
            name: name.to_string(),
            is_update_fn: true,
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

impl fmt::Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quantifier::Exists => write!(f, "\\exists"),
            Quantifier::Forall => write!(f, "\\forall"),
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Atom::Var(name) => write!(f, "{name}"),
            Atom::True => write!(f, "1"),
            Atom::False => write!(f, "0"),
        }
    }
}

impl fmt::Display for FunctionSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<bool> for Atom {
    fn from(value: bool) -> Self {
        if value {
            Atom::True
        } else {
            Atom::False
        }
    }
}
