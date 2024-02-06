use crate::sketchbook::layout::{Layout, LayoutId};

/// **(internal)** Utility methods for `Essentiality`.
mod _essentiality;
/// **(internal)** Utility methods for `Identifier`.
mod _identifier;
/// **(internal)** Utility methods for `ModelState`.
mod _model_state;
/// **(internal)** Utility methods for `Parameter`.
mod _parameter;
/// **(internal)** Utility methods for `ParameterId`.
mod _parameter_id;
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
/// Classes and utility methods that can be used for sending simplified data to frontend.
/// This includes simplified "data carriers" for variables, regulations, and layouts.
pub mod simplified_structs;

pub use _essentiality::Essentiality;
pub use _identifier::Identifier;
pub use _model_state::ModelState;
pub use _parameter::Parameter;
pub use _parameter_id::ParamId;
pub use _regulation::Regulation;
pub use _regulation_sign::RegulationSign;
pub use _var_id::VarId;
pub use _variable::Variable;

/// An iterator over all (`VarId`, `Variable`) pairs of a `ModelState`.
pub type VariableIterator<'a> = std::collections::hash_map::Keys<'a, VarId, Variable>;

/// An iterator over all (`ParamId`, `Parameter`) pairs of a `ModelState`.
pub type ParameterIterator<'a> = std::collections::hash_map::Keys<'a, ParamId, Parameter>;

/// An iterator over all `Regulations` of a `ModelState`.
pub type RegulationIterator<'a> = std::collections::hash_set::Iter<'a, Regulation>;

/// An iterator over all (`LayoutId`, `Layout`) pairs of a `ModelState`.
pub type LayoutIterator<'a> = std::collections::hash_map::Keys<'a, LayoutId, Layout>;
