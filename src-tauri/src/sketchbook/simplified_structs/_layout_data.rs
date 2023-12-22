use crate::sketchbook::layout::Layout;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Structure for sending simplified data about `Layout` to frontend.
/// Only contains some fields, in string format, to allow for simpler parsing and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutData {
    pub id: String,
    pub name: String,
}

impl LayoutData {
    pub fn new(id: String, name: String) -> LayoutData {
        LayoutData { id, name }
    }

    pub fn from_layout(id: String, layout: &Layout) -> LayoutData {
        LayoutData {
            id,
            name: layout.get_layout_name().to_string(),
        }
    }
}

impl Display for LayoutData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
