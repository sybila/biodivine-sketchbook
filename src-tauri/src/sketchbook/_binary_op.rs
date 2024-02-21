use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Possible binary Boolean operators that can appear in `FnUpdate`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    And,
    Or,
    Xor,
    Iff,
    Imp,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let symbol = match self {
            BinaryOp::And => "&",
            BinaryOp::Or => "|",
            BinaryOp::Xor => "^",
            BinaryOp::Imp => "=>",
            BinaryOp::Iff => "<=>",
        };
        write!(f, "{}", symbol)
    }
}
