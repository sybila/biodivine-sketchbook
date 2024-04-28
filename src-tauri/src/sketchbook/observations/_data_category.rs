use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Enum of possible categories of observation datasets.
/// The category may depend on how were the data measured, or how we'd like to interpret it.
///
/// Defining the category enables us to automatically encode observation lists via temporal formulae.
/// If the category is unknown, or is not covered, use [DataCategory::Unspecified].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Copy)]
pub enum DataCategory {
    Attractor,
    FixedPoint,
    TimeSeries,
    Unspecified,
}

impl<'de> JsonSerde<'de> for DataCategory {}

impl fmt::Display for DataCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataCategory::Attractor => write!(f, "Attractor"),
            DataCategory::FixedPoint => write!(f, "FixedPoint"),
            DataCategory::TimeSeries => write!(f, "TimeSeries"),
            DataCategory::Unspecified => write!(f, "Unspecified"),
        }
    }
}
