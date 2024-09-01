use crate::sketchbook::ids::{StatPropertyId, VarId};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::StatProperty;

/// Few commonly used utilities regarding static properties.
/// In future, we might consider moving this elsewhere.

/// Get ID of a static property that describes monotonicity of a regulation
/// between `regulator` and `target`.
pub fn get_monotonicity_prop_id(regulator: &VarId, target: &VarId) -> StatPropertyId {
    let id_str = format!("monotonicity_{}_{}", regulator, target);
    // this will always be a valid ID string, we can unwrap
    StatPropertyId::new(&id_str).unwrap()
}

/// Get ID of a static property that describes essentiality of a regulation
/// between `regulator` and `target`.
pub fn get_essentiality_prop_id(regulator: &VarId, target: &VarId) -> StatPropertyId {
    let id_str = format!("essentiality_{}_{}", regulator, target);
    // this will always be a valid ID string, we can unwrap
    StatPropertyId::new(&id_str).unwrap()
}

/// Shorthand to get a static property that describes essentiality of a regulation
/// between `regulator` and `target`.
pub fn get_essentiality_prop(
    regulator: &VarId,
    target: &VarId,
    essentiality: Essentiality,
) -> StatProperty {
    let name_str = "Regulation essentiality property".to_string();
    // this will always be a valid name string, we can unwrap
    StatProperty::mk_regulation_essential(
        &name_str,
        Some(regulator.clone()),
        Some(target.clone()),
        essentiality,
    )
    .unwrap()
}

/// **(internal)** Shorthand to get a static property that describes monotonicity of a regulation
/// between `regulator` and `target`.
pub fn get_monotonicity_prop(
    regulator: &VarId,
    target: &VarId,
    monotonicity: Monotonicity,
) -> StatProperty {
    let name_str = "Regulation monotonicity property".to_string();
    // this will always be a valid name string, we can unwrap
    StatProperty::mk_regulation_monotonic(
        &name_str,
        Some(regulator.clone()),
        Some(target.clone()),
        monotonicity,
    )
    .unwrap()
}
