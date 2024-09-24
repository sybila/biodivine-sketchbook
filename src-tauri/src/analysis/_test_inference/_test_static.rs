use super::utils::{load_test_model, run_inference};
use crate::sketchbook::model::Monotonicity;
use crate::sketchbook::properties::StatProperty;

#[test]
/// Test inference using the test model with added monotonicity properties in FOL.
fn inference_fol_monotonicity_activation() {
    let mut sketch = load_test_model();
    let formula = "f_D(0) => f_D(1)";
    let id = "d_d_is_activation";
    let property = StatProperty::mk_generic(id, formula).unwrap();
    sketch
        .properties
        .add_raw_static_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 16);
}

#[test]
/// Test inference using the test model with added monotonicity properties in FOL.
fn inference_fol_monotonicity_dual() {
    let mut sketch = load_test_model();
    let formula = "!(f_D(0) => f_D(1)) & !(f_D(1) => f_D(0))";
    let id = "d_d_is_dual";
    let property = StatProperty::mk_generic(id, formula).unwrap();
    sketch
        .properties
        .add_raw_static_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 0);
}

#[test]
/// Test inference using the test model with added template monotonicity properties.
fn inference_template_monotonicity_activation() {
    let mut sketch = load_test_model();
    let var_d = sketch.model.get_var_id("D").unwrap();
    let id = "d_d_is_activation";
    let property = StatProperty::mk_regulation_monotonic(
        id,
        Some(var_d.clone()),
        Some(var_d.clone()),
        Monotonicity::Activation,
    )
    .unwrap();
    sketch
        .properties
        .add_raw_static_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 16);
}

#[test]
/// Test inference using the test model with added template monotonicity properties.
fn inference_template_monotonicity_dual() {
    let mut sketch = load_test_model();
    let var_d = sketch.model.get_var_id("D").unwrap();
    let id = "d_d_is_dual";
    let property = StatProperty::mk_regulation_monotonic(
        id,
        Some(var_d.clone()),
        Some(var_d.clone()),
        Monotonicity::Dual,
    )
    .unwrap();
    sketch
        .properties
        .add_raw_static_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 0);
}
