use crate::sketchbook::ids::{LayoutId, UninterpretedFnId, VarId};
use crate::sketchbook::layout::Layout;

/// **(internal)** Definition and utility methods for `BinaryOp`.
mod _binary_op;
/// **(internal)** Definition and utility methods for `Essentiality`.
mod _essentiality;
/// **(internal)** Definition and utility methods for `FnTree`.
mod _function_tree;
/// **(internal)** Definition and utility methods for the manager class `ModelState`.
mod _model_state;
/// **(internal)** Definition and utility methods for `Monotonicity`.
mod _monotonicity;
/// **(internal)** Definition and utility methods for `Regulation`.
mod _regulation;
/// **(internal)** Definition and utility methods for `UninterpretedFn`.
mod _uninterpreted_fn;
/// **(internal)** Definition and utility methods for `FnArgument`.
mod _uninterpreted_fn_arg;
/// **(internal)** Definition and utility methods for `UpdateFn`.
mod _update_function;
/// **(internal)** Definition and utility methods for `Variable`.
mod _variable;

pub use _binary_op::BinaryOp;
pub use _essentiality::Essentiality;
pub use _function_tree::FnTree;
pub use _model_state::ModelState;
pub use _monotonicity::Monotonicity;
pub use _regulation::Regulation;
pub use _uninterpreted_fn::UninterpretedFn;
pub use _uninterpreted_fn_arg::FnArgument;
pub use _update_function::UpdateFn;
pub use _variable::Variable;

/// An iterator over all (`VarId`, `Variable`) pairs of a `ModelState`.
pub type VariableIterator<'a> = std::collections::hash_map::Iter<'a, VarId, Variable>;

/// An iterator over all (`VarId`, `UpdateFn`) pairs of a `ModelState`.
pub type UpdateFnIterator<'a> = std::collections::hash_map::Iter<'a, VarId, UpdateFn>;

/// An iterator over all (`UninterpretedFnId`, `UninterpretedFn`) pairs of a `ModelState`.
pub type UninterpretedFnIterator<'a> =
    std::collections::hash_map::Iter<'a, UninterpretedFnId, UninterpretedFn>;

/// An iterator over all `Regulations` of a `ModelState`.
pub type RegulationIterator<'a> = std::collections::hash_set::Iter<'a, Regulation>;

/// An iterator over all (`LayoutId`, `Layout`) pairs of a `ModelState`.
pub type LayoutIterator<'a> = std::collections::hash_map::Iter<'a, LayoutId, Layout>;
