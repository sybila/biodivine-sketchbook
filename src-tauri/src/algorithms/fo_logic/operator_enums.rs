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

/// Enum for terms: variables and constants.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Term {
    Function(String, Vec<(bool, Term)>), // A function with a list of (potentially negated) args
    Var(String),                         // A variable name
    True,                                // A true constant
    False,                               // A false constant
}

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

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Term::Var(name) => write!(f, "{name}"),
            Term::True => write!(f, "1"),
            Term::False => write!(f, "0"),
            Term::Function(name, arguments) => {
                let formatted_args: Vec<String> = arguments
                    .iter()
                    .map(|(negate, term)| {
                        if *negate {
                            format!("!{term}")
                        } else {
                            format!("{term}")
                        }
                    })
                    .collect();
                write!(f, "{}({})", name, formatted_args.join(", "))
            }
        }
    }
}

impl From<bool> for Term {
    fn from(value: bool) -> Self {
        if value {
            Term::True
        } else {
            Term::False
        }
    }
}
