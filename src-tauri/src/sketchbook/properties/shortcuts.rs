use crate::sketchbook::ids::{UninterpretedFnId, VarId};
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
pub fn mk_reg_essentiality_prop(
    regulator: &VarId,
    target: &VarId,
    essentiality: Essentiality,
) -> StatProperty {
    let name_str = "Regulation essentiality (generated)";
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
pub fn mk_reg_monotonicity_prop(
    regulator: &VarId,
    target: &VarId,
    monotonicity: Monotonicity,
) -> StatProperty {
    let name_str = "Regulation monotonicity (generated)";
    StatProperty::mk_regulation_monotonic(
        name_str,
        Some(regulator.clone()),
        Some(target.clone()),
        monotonicity,
        "",
    )
}

/// Shorthand to get a static property that describes that argument of an
/// uninterpreted function `fn_id` on index `index` is essential.
/// Chosen name is generic and annotation is empty.
pub fn mk_fn_input_essentiality_prop(
    fn_id: &UninterpretedFnId,
    index: usize,
    essentiality: Essentiality,
) -> StatProperty {
    let name_str = "Fn input essentiality (generated)";
    StatProperty::mk_fn_input_essential(
        name_str,
        Some(index),
        Some(fn_id.clone()),
        essentiality,
        "",
    )
}

/// Shorthand to get a static property that describes that argument of an
/// uninterpreted function `fn_id` on index `index` is monotonic.
/// Chosen name is generic and annotation is empty.
pub fn mk_fn_input_monotonicity_prop(
    fn_id: &UninterpretedFnId,
    index: usize,
    monotonicity: Monotonicity,
) -> StatProperty {
    let name_str = "Fn input monotonicity (generated)";
    StatProperty::mk_fn_input_monotonic(
        name_str,
        Some(index),
        Some(fn_id.clone()),
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
