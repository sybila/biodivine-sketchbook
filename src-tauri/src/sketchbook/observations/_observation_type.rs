use serde::{Deserialize, Serialize};
use std::fmt;

/// Enum of possible types of observation datasets.
/// The type may depend on how were the data measured, or how we'd like to interpret it.
///
/// Defining the type enables us to automatically encode observation lists via temporal formulae.
/// If the type is unknown, or is not covered, use [ObservationType::Unspecified].
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, Copy)]
pub enum ObservationType {
    Attractor,
    FixedPoint,
    TimeSeries,
    Unspecified,
}

impl fmt::Display for ObservationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObservationType::Attractor => write!(f, "Attractor"),
            ObservationType::FixedPoint => write!(f, "FixedPoint"),
            ObservationType::TimeSeries => write!(f, "TimeSeries"),
            ObservationType::Unspecified => write!(f, "Unspecified"),
        }
    }
}
