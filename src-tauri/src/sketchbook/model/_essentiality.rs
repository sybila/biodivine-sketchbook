use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Possible variants of essentiality of a `Regulation`.
///
/// - `True` means that the regulation is essential and must have an effect
/// - `False` means that it has no effect
/// - `Unknown` means it might or might not have an effect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Essentiality {
    True,
    False,
    Unknown,
}

impl JsonSerde<'_> for Essentiality {}
