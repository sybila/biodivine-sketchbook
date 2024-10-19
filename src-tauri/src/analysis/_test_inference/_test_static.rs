use super::utils::load_test_model;
use crate::analysis::_test_inference::utils::add_stat_prop_and_infer;
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::shortcuts::*;
use crate::sketchbook::properties::StatProperty;

#[test]
/// Test inference using the test model with added monotonicity properties in FOL.
fn inference_fol_monotonicity() {
    // activation D -> D
    let sketch = load_test_model();
    let formula = "f_D(0) => f_D(1)";
    let id = "d_d_is_activation";
    let property = mk_fol_prop(formula).unwrap();
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 16);

    // inhibition D -| D
    let sketch = load_test_model();
    let formula = "f_D(1) => f_D(0)";
    let id = "d_d_is_inhibition";
    let property = mk_fol_prop(formula).unwrap();
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 16);

    // dual D -* D
    let sketch = load_test_model();
    let formula = "!(f_D(0) => f_D(1)) & !(f_D(1) => f_D(0))";
    let id = "d_d_is_dual";
    let property = mk_fol_prop(formula).unwrap();
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 0);
}

#[test]
/// Test inference using the test model with added template monotonicity properties.
fn inference_template_monotonicity() {
    let sketch = load_test_model();
    let var_d = sketch.model.get_var_id("D").unwrap();
    let id = "d_d_is_activation";
    let property = mk_monotonicity_prop(&var_d, &var_d, Monotonicity::Activation);
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 16);

    let sketch = load_test_model();
    let var_d = sketch.model.get_var_id("D").unwrap();
    let id = "d_d_is_inhibition";
    let property = mk_monotonicity_prop(&var_d, &var_d, Monotonicity::Inhibition);
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 16);

    let sketch = load_test_model();
    let var_d = sketch.model.get_var_id("D").unwrap();
    let id = "d_d_is_dual";
    let property = mk_monotonicity_prop(&var_d, &var_d, Monotonicity::Dual);
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 0);
}

#[test]
/// Test inference using the test model with added template essentiality properties.
fn inference_fol_essentiality() {
    let sketch = load_test_model();
    let formula = "f_A(1) ^ f_A(0)";
    let id = "c_a_is_essential";
    let property = StatProperty::try_mk_generic(id, formula, "").unwrap();
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 16);

    let sketch = load_test_model();
    let formula = "!(f_A(1) ^ f_A(0))";
    let id = "c_a_not_essential";
    let property = StatProperty::try_mk_generic(id, formula, "").unwrap();
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 16);
}

#[test]
/// Test inference using the test model with added template essentiality properties.
fn inference_template_essentiality() {
    let sketch = load_test_model();
    let var_c = sketch.model.get_var_id("C").unwrap();
    let var_a = sketch.model.get_var_id("A").unwrap();
    let id = "c_a_is_essential";
    let property = mk_essentiality_prop(&var_c, &var_a, Essentiality::True);
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 16);

    let sketch = load_test_model();
    let var_c = sketch.model.get_var_id("C").unwrap();
    let var_a = sketch.model.get_var_id("A").unwrap();
    let id = "c_a_not_essential";
    let property = mk_essentiality_prop(&var_c, &var_a, Essentiality::False);
    assert_eq!(add_stat_prop_and_infer(sketch, property, id), 16);
}
