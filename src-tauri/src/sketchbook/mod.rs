use crate::sketchbook::layout::{Layout, LayoutId};

/// **(internal)** Utility methods for `Identifier`.
mod _identifier;
/// **(internal)** Utility methods for `ModelState`.
mod _model_state;
/// **(internal)** Utility methods for `Regulation`.
mod _regulation;
/// **(internal)** Utility methods for `RegulationSign`.
mod _regulation_sign;
/// **(internal)** Utility methods for `VarId`.
mod _var_id;
/// **(internal)** Utility methods for `Variable`.
mod _variable;

/// Classes and utility methods regarding the layout of the Regulations editor.
pub mod layout;

pub use _identifier::Identifier;
pub use _model_state::ModelState;
pub use _regulation::Regulation;
pub use _regulation_sign::RegulationSign;
pub use _var_id::VarId;
pub use _variable::Variable;

/// An iterator over all (`VarId`, `Variable`) pairs of a `ModelState`.
pub type VariableIterator<'a> = std::collections::hash_map::Keys<'a, VarId, Variable>;

/// An iterator over all `Regulations` of a `ModelState`.
pub type RegulationIterator<'a> = std::collections::hash_set::Iter<'a, Regulation>;

/// An iterator over all (`LayoutId`, `Layout`) pairs of a `ModelState`.
pub type LayoutIterator<'a> = std::collections::hash_map::Keys<'a, LayoutId, Layout>;
