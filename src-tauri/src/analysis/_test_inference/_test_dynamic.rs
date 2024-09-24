use super::utils::{load_test_model, run_inference};
use crate::sketchbook::properties::DynProperty;

#[test]
/// Test inference using the test model with added properties in HCTL.
fn inference_hctl_multi_attrs() {
    let mut sketch = load_test_model();
    let formula = "3{x}: (3{y}: (@{x}: (AG~{y}) & (AG EF {x})) & (@{y}: AG EF {y}))";
    let id = "at_least_2_attrs";
    let property = DynProperty::mk_generic(id, formula).unwrap();
    sketch
        .properties
        .add_raw_dynamic_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 17);
}

#[test]
/// Test inference using the test model with added properties in HCTL.
fn inference_hctl_multi_fixed_points() {
    let mut sketch = load_test_model();
    let formula = "3{x}: (3{y}: (@{x}: ~{y} & (AX {x})) & (@{y}: AX {y}))";
    let id = "at_least_2_attrs";
    let property = DynProperty::mk_generic(id, formula).unwrap();
    sketch
        .properties
        .add_raw_dynamic_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 9);
}

#[test]
/// Test inference using the test model with added attr count template properties.
fn inference_template_attr_count() {
    let mut sketch = load_test_model();
    let id = "exactly_1_attr";
    let property = DynProperty::mk_attractor_count(id, 1, 1).unwrap();
    sketch
        .properties
        .add_raw_dynamic_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 15);

    let mut sketch = load_test_model();
    let id = "exactly_3_attr";
    let property = DynProperty::mk_attractor_count(id, 3, 3).unwrap();
    sketch
        .properties
        .add_raw_dynamic_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 1);

    let mut sketch = load_test_model();
    let id = "range_3_4_attr";
    let property = DynProperty::mk_attractor_count(id, 3, 4).unwrap();
    sketch
        .properties
        .add_raw_dynamic_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 2);

    let mut sketch = load_test_model();
    let id = "range_5_attr";
    let property = DynProperty::mk_attractor_count(id, 5, 5).unwrap();
    sketch
        .properties
        .add_raw_dynamic_by_str(id, property)
        .unwrap();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 0);
}
