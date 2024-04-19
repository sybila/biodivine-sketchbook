use crate::sketchbook::ids::VarId;
use crate::sketchbook::layout::LayoutNode;
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// **(internal)** Basic utility methods for `Layout`.
mod _impl_layout;
/// **(internal)** Implementation of [Serialize] and [Deserialize] traits for `Layout`.
mod _impl_layout_serde;

/// Structure to capture all the layout data regarding one particular layout of the regulations
/// editor.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    name: String,
    nodes: HashMap<VarId, LayoutNode>,
}

impl Display for Layout {
    /// Use json serialization to convert `Layout` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for Layout {
    type Err = String;

    /// Use json de-serialization to construct `Layout` from string.
    fn from_str(s: &str) -> Result<Layout, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
