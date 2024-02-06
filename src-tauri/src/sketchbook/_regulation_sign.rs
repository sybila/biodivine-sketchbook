use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Possible variants of (non)-monotonous effects of a `Regulation`.
///
/// - `Activation` means positive monotonicity
/// - `Inhibition` means negative monotonicity
/// - `Dual` means both positive and negative effect
/// - `Unknown` stands for unknown effect
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum RegulationSign {
    Activation,
    Inhibition,
    Dual,
    Unknown,
}

impl Display for RegulationSign {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let sign_str = self.as_str_shortcut();
        write!(f, "{sign_str}")
    }
}

impl RegulationSign {
    pub fn try_from_str_shortcut(sign: &str) -> Result<RegulationSign, String> {
        match sign {
            "?" => Ok(RegulationSign::Unknown),
            ">" => Ok(RegulationSign::Activation),
            "|" => Ok(RegulationSign::Inhibition),
            "*" => Ok(RegulationSign::Dual),
            _ => Err(format!("{} does not encode any `RegulationSign`", sign)),
        }
    }

    pub fn try_from_str_full(sign: &str) -> Result<RegulationSign, String> {
        match sign {
            "Unknown" => Ok(RegulationSign::Unknown),
            "Activation" => Ok(RegulationSign::Activation),
            "Inhibition" => Ok(RegulationSign::Inhibition),
            "Dual" => Ok(RegulationSign::Dual),
            _ => Err(format!("{} does not describe any `RegulationSign`", sign)),
        }
    }

    pub fn as_str_shortcut(&self) -> &str {
        match self {
            RegulationSign::Unknown => "?",
            RegulationSign::Activation => ">",
            RegulationSign::Inhibition => "|",
            RegulationSign::Dual => "*",
        }
    }

    pub fn as_str_full(&self) -> &str {
        match self {
            RegulationSign::Unknown => "Unknown",
            RegulationSign::Activation => "Activation",
            RegulationSign::Inhibition => "Inhibition",
            RegulationSign::Dual => "Dual",
        }
    }
}
