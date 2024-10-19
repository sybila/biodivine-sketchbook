use crate::sketchbook::ids::VarId;
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::StatProperty;

use super::DynProperty;

/// Shorthand to get a generic static property with FOL formula.
/// Chosen name is generic and annotation is empty.
pub fn mk_fol_prop(formula: &str) -> Result<StatProperty, String> {
    let name_str = "Generic FOL property";
    StatProperty::try_mk_generic(name_str, formula, "")
}

/// Shorthand to get a static property that describes essentiality of a regulation
/// between `regulator` and `target`.
/// Chosen name is generic and annotation is empty.
pub fn mk_essentiality_prop(
    regulator: &VarId,
    target: &VarId,
    essentiality: Essentiality,
) -> StatProperty {
    let name_str = "Regulation essentiality property";
    StatProperty::mk_regulation_essential(
        name_str,
        Some(regulator.clone()),
        Some(target.clone()),
        essentiality,
        "",
    )
}

/// Shorthand to get a static property that describes monotonicity of a regulation
/// between `regulator` and `target`.
/// Chosen name is generic and annotation is empty.
pub fn mk_monotonicity_prop(
    regulator: &VarId,
    target: &VarId,
    monotonicity: Monotonicity,
) -> StatProperty {
    let name_str = "Regulation monotonicity property";
    StatProperty::mk_regulation_monotonic(
        name_str,
        Some(regulator.clone()),
        Some(target.clone()),
        monotonicity,
        "",
    )
}

/// Shorthand to get a generic dynamic property with HCTL formula.
/// Chosen name is generic and annotation is empty.
pub fn mk_hctl_prop(formula: &str) -> Result<DynProperty, String> {
    let name_str = "Generic HCTL property";
    DynProperty::try_mk_generic(name_str, formula, "")
}
