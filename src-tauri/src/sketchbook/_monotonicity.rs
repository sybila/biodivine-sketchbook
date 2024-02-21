use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Possible variants of (non)-monotonous effects of a `Regulation`.
///
/// - `Activation` means positive monotonicity
/// - `Inhibition` means negative monotonicity
/// - `Dual` means both positive and negative effect
/// - `Unknown` stands for unknown effect
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Monotonicity {
    Activation,
    Inhibition,
    Dual,
    Unknown,
}

impl Display for Monotonicity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let sign_str = self.as_str_shortcut();
        write!(f, "{sign_str}")
    }
}

impl Monotonicity {
    pub fn try_from_str_shortcut(sign: &str) -> Result<Monotonicity, String> {
        match sign {
            "?" => Ok(Monotonicity::Unknown),
            ">" => Ok(Monotonicity::Activation),
            "|" => Ok(Monotonicity::Inhibition),
            "*" => Ok(Monotonicity::Dual),
            _ => Err(format!("{} does not encode any `Monotonicity`", sign)),
        }
    }

    pub fn try_from_str_full(sign: &str) -> Result<Monotonicity, String> {
        match sign {
            "Unknown" => Ok(Monotonicity::Unknown),
            "Activation" => Ok(Monotonicity::Activation),
            "Inhibition" => Ok(Monotonicity::Inhibition),
            "Dual" => Ok(Monotonicity::Dual),
            _ => Err(format!("{} does not describe any `Monotonicity`", sign)),
        }
    }

    pub fn as_str_shortcut(&self) -> &str {
        match self {
            Monotonicity::Unknown => "?",
            Monotonicity::Activation => ">",
            Monotonicity::Inhibition => "|",
            Monotonicity::Dual => "*",
        }
    }

    pub fn as_str_full(&self) -> &str {
        match self {
            Monotonicity::Unknown => "Unknown",
            Monotonicity::Activation => "Activation",
            Monotonicity::Inhibition => "Inhibition",
            Monotonicity::Dual => "Dual",
        }
    }
}
