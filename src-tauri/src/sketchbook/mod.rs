use crate::sketchbook::layout::{Layout, LayoutId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Classes and utility methods regarding the layout of the Regulations editor.
pub mod layout;

/// **(internal)** Utility methods for `Identifier`.
mod _impl_identifier;
/// **(internal)** Utility methods for `Regulation`.
mod _impl_regulation;
/// **(internal)** Utility methods for `RegulationState`.
mod _impl_regulations_state;
/// **(internal)** Utility methods for `VarId`.
mod _impl_var_id;
/// **(internal)** Utility methods for `Variable`.
mod _impl_variable;

/// A type-safe identifier that can be used for IDs of various objects, such as of variables
/// (see `VarId`) or layouts (see `LayoutId`). Corresponds to a C-like identifier, or SBML's SId.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Identifier {
    id: String,
}

/// A type-safe (string-based) identifier of a `Variable` inside `RegulationsState`.
///
/// **Warning:** Do not mix identifiers between different networks/graphs. Generally, be careful
/// to only use `VarIds` currently valid for the network.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct VarId {
    id: Identifier,
}

/// A type safe object for a Boolean variable of a `RegulationsState`.
///
/// Currently, it only stores the variable's `name`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Variable {
    name: String,
}

/// Possible variants of (non)-monotonous effects of a `Regulation`.
/// `Activation` means positive and `Inhibition` means negative monotonicity, `Dual` means both
/// positive and negative effect, `Unknown` for unknown effect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum RegulationSign {
    Activation,
    Inhibition,
    Dual,
    Unknown,
}

/// Describes an interaction between two variables, `regulator` and `target`.
/// Every regulation can be *monotonous* and can be set as *observable*:
///
///  - Monotonicity is either *positive* or *negative* and signifies that the influence of the
/// `regulator` on the `target` has to *increase* or *decrease* the `target` value respectively.
///  - If observability is set to `true`, the `regulator` *must* have influence on the outcome
///  of the `target` update function in *some* context. If set to false, this is not enforced
///  (i.e. the `regulator` *can* have an influence on the `target`, but it is not required).
///
/// Regulations can be represented as strings in the
/// form `"regulator_name 'relationship' target_name"`. The 'relationship' starts with `-`, which
/// is followed by `>` for activation (positive monotonicity), `|` for inhibition (negative
/// monotonicity), `D` for dual effect (non-monotonic) or `?` for unspecified monotonicity.
/// Finally, an additional `?` at the end of 'relationship' signifies a non-observable
/// (non-essential) regulation.
/// Together, this gives the following options:  `->, ->?, -|, -|?, -D, -D?, -?, -??`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Regulation {
    regulator: VarId,
    target: VarId,
    observable: bool,
    regulation_sign: RegulationSign,
}

/// Object representing the state of the Regulations editor.
///
/// Holds similar information as `RegulatoryGraph` in `lib-param-bn`, but is suitable for
/// editing. Further, the information regarding the layout is carried.
#[derive(Clone, Debug)]
pub struct RegulationsState {
    variables: HashMap<VarId, Variable>,
    regulations: HashSet<Regulation>,
    layouts: HashMap<LayoutId, Layout>,
}

/// An iterator over all (`VarId`, `Variable`) pairs of a `RegulationsState`.
pub type VariableIterator<'a> = std::collections::hash_map::Keys<'a, VarId, Variable>;

/// An iterator over all `Regulations` of a `RegulationsState`.
pub type RegulationIterator<'a> = std::collections::hash_set::Iter<'a, Regulation>;

/// An iterator over all (`LayoutId`, `Layout`) pairs of a `RegulationsState`.
pub type LayoutIterator<'a> = std::collections::hash_map::Keys<'a, LayoutId, Layout>;
