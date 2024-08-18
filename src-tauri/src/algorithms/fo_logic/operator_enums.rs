use std::fmt;

/// Enum for all possible Boolean unary operators occurring in a FOL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum UnaryOp {
    Not, // '!'
}

/// Enum for all possible Boolean binary operators occurring in a FOL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum BinaryOp {
    And, // '&'
    Or,  // '|'
    Xor, // '^'
    Imp, // '=>'
    Iff, // '<=>'
}

/// Enum for quantifiers in a FOL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
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

// A function symbol
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FunctionSymbol(pub String);

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinaryOp::And => write!(f, "&"),
            BinaryOp::Or => write!(f, "|"),
            BinaryOp::Xor => write!(f, "^"),
            BinaryOp::Imp => write!(f, "=>"),
            BinaryOp::Iff => write!(f, "<=>"),
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
        write!(f, "{}", self.0)
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
