use biodivine_lib_param_bn::ExtendedBoolean;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Enum of possible values of network variables in each observation.
/// We consider binary values and unspecified variant.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum VarValue {
    True,
    False,
    Any,
}

impl fmt::Display for VarValue {
    /// Transform the value to one of the `1`, `0`, or `*`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for VarValue {
    type Err = String;

    /// Try to parse the value. Valid strings are one of the `1`, `0`, or `*`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(VarValue::True),
            "0" => Ok(VarValue::False),
            "*" => Ok(VarValue::Any),
            _ => Err("Invalid value string.".to_string()),
        }
    }
}

impl From<bool> for VarValue {
    fn from(value: bool) -> Self {
        if value {
            VarValue::True
        } else {
            VarValue::False
        }
    }
}

impl From<Option<bool>> for VarValue {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(value) => VarValue::from(value),
            None => VarValue::Any,
        }
    }
}

impl From<ExtendedBoolean> for VarValue {
    fn from(value: ExtendedBoolean) -> Self {
        match value {
            ExtendedBoolean::One => VarValue::True,
            ExtendedBoolean::Zero => VarValue::False,
            ExtendedBoolean::Any => VarValue::Any,
        }
    }
}

impl VarValue {
    /// Return `true` value if is not specified.
    pub fn is_any(&self) -> bool {
        *self == VarValue::Any
    }

    /// Return `true` value if is specified.
    pub fn is_fixed(&self) -> bool {
        *self != VarValue::Any
    }

    /// Return Boolean if value is specified, else None.
    pub fn try_as_bool(&self) -> Option<bool> {
        match self {
            VarValue::True => Some(true),
            VarValue::False => Some(false),
            VarValue::Any => None,
        }
    }

    /// Return a string slice encoding this value (one of the `1`, `0`, or `*`).
    pub fn as_str(&self) -> &str {
        match self {
            VarValue::True => "1",
            VarValue::False => "0",
            VarValue::Any => "*",
        }
    }
}
