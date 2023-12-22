use crate::sketchbook::layout::{Layout, LayoutId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending simplified data about `Layout` to frontend.
/// Only contains some fields, in string format, to allow for simpler parsing and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutData {
    pub id: String,
    pub name: String,
}

impl LayoutData {
    pub fn new(layout_id: &LayoutId, layout_name: &str) -> LayoutData {
        LayoutData {
            id: layout_id.to_string(),
            name: layout_name.to_string(),
        }
    }

    pub fn from_layout(layout_id: &LayoutId, layout: &Layout) -> LayoutData {
        LayoutData {
            id: layout_id.to_string(),
            name: layout.get_layout_name().to_string(),
        }
    }
}

impl Display for LayoutData {
    /// Use json serialization to convert `LayoutData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for LayoutData {
    type Err = String;

    /// Use json de-serialization to construct `LayoutData` from string.
    fn from_str(s: &str) -> Result<LayoutData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
