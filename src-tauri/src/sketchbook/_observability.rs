use serde::{Deserialize, Serialize};

/// Possible variants of observability of a `Regulation`.
///
/// - `True` means that the regulation is observable and must have an effect
/// - `False` means that it has no effect
/// - `Unknown` means it might or might not have an effect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Observability {
    True,
    False,
    Unknown,
}
