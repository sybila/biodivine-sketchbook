use super::utils::load_test_model;
use crate::inference::_test_inference::utils::add_dyn_prop_and_infer;
use crate::sketchbook::properties::shortcuts::*;
use crate::sketchbook::properties::DynProperty;

#[test]
/// Test inference using the test model with added properties in HCTL.
fn inference_hctl() {
    // Multiple attractors
    let sketch = load_test_model();
    let formula = "3{x}: (3{y}: (@{x}: (AG~{y}) & (AG EF {x})) & (@{y}: AG EF {y}))";
    let id = "at_least_2_attrs";
    let property = mk_hctl_prop(formula).unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 17);

    // Multiple fixed-points
    let sketch = load_test_model();
    let formula = "3{x}: (3{y}: (@{x}: ~{y} & (AX {x})) & (@{y}: AX {y}))";
    let id = "at_least_2_attrs";
    let property = mk_hctl_prop(formula).unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 9);

    // No fixed-points
    let sketch = load_test_model();
    let formula = "~(3{x}: @{x}: AX {x})";
    let id = "at_least_2_attrs";
    let property = mk_hctl_prop(formula).unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 20);

    // Has 1111 fixed point
    let sketch = load_test_model();
    let formula = "3{x}: @{x}: (A & B & C & D & AX {x})";
    let id = "has_1111_fixed_point";
    let property = mk_hctl_prop(formula).unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 4);

    // State 1111 is part of an attractor
    let sketch = load_test_model();
    let formula = "3{x}: @{x}: (A & B & C & D & AG EF {x})";
    let id = "has_1111_in_attractor";
    let property = mk_hctl_prop(formula).unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 19);

    // Has 111* minimal trap space
    let sketch = load_test_model();
    let formula = "V{x}: @{x}: (((A & B & C) => AG (A & B & C)) & ~ AX {x})";
    let id = "has_111X_mts";
    let property = mk_hctl_prop(formula).unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 2);
}

#[test]
/// Test inference using the test model with added attr count template properties.
fn inference_template_attr_count() {
    let sketch = load_test_model();
    let id = "exactly_1_attr";
    let property = DynProperty::try_mk_attractor_count(id, 1, 1, "").unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 15);

    let sketch = load_test_model();
    let id = "exactly_3_attr";
    let property = DynProperty::try_mk_attractor_count(id, 3, 3, "").unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 1);

    let sketch = load_test_model();
    let id = "range_3_4_attr";
    let property = DynProperty::try_mk_attractor_count(id, 3, 4, "").unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 2);

    let sketch = load_test_model();
    let id = "range_5_attr";
    let property = DynProperty::try_mk_attractor_count(id, 5, 5, "").unwrap();
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 0);
}

#[test]
/// Test inference using the test model with added fixed-point template properties.
fn inference_template_fixed_point() {
    // Has 1111 fixed point
    let sketch = load_test_model();
    let id = "has_1111_fixed_point";
    let data_id = sketch.observations.get_dataset_id("data_fp").unwrap();
    let obs_id = sketch.observations.get_obs_id("data_fp", "ones").unwrap();
    let property = DynProperty::mk_fixed_point(id, Some(data_id), Some(obs_id), "");
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 4);
}

#[test]
/// Test inference using the test model with added attractor template properties.
fn inference_template_attractor() {
    // State 1111 is part of an attractor
    let sketch = load_test_model();
    let id = "has_1111_in_attractor";
    let data_id = sketch.observations.get_dataset_id("data_fp").unwrap();
    let obs_id = sketch.observations.get_obs_id("data_fp", "ones").unwrap();
    let property = DynProperty::mk_has_attractor(id, Some(data_id), Some(obs_id), "");
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 19);
}

#[test]
/// Test inference using the test model with added trap-space template properties.
fn inference_template_trap_space() {
    // Has 111* (general) trap space
    let sketch = load_test_model();
    let id = "has_111X_ts";
    let data_id = sketch.observations.get_dataset_id("data_mts").unwrap();
    let obs_id = sketch.observations.get_obs_id("data_mts", "abc").unwrap();
    let property = DynProperty::mk_trap_space(id, Some(data_id), Some(obs_id), false, false, "");
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 4);

    // Has 111* essential trap space
    let sketch = load_test_model();
    let id = "has_111X_ets";
    let data_id = sketch.observations.get_dataset_id("data_mts").unwrap();
    let obs_id = sketch.observations.get_obs_id("data_mts", "abc").unwrap();
    let property = DynProperty::mk_trap_space(id, Some(data_id), Some(obs_id), false, true, "");
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 4);

    // Has 111* minimal trap space
    let sketch = load_test_model();
    let id = "has_111X_mts";
    let data_id = sketch.observations.get_dataset_id("data_mts").unwrap();
    let obs_id = sketch.observations.get_obs_id("data_mts", "abc").unwrap();
    let property = DynProperty::mk_trap_space(id, Some(data_id), Some(obs_id), true, true, "");
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 2);
}

#[test]
/// Test inference using the test model with added trajectory template properties.
fn inference_template_time_series() {
    // There is a trajectory 1000 -> 1100 -> 1110 -> 1111
    let sketch = load_test_model();
    let id = "time_serie";
    let data_id = sketch
        .observations
        .get_dataset_id("data_time_series")
        .unwrap();
    let property = DynProperty::mk_trajectory(id, Some(data_id), "");
    assert_eq!(add_dyn_prop_and_infer(sketch, property, id), 16);
}