use crate::analysis::_test_inference::utils::apply_event_fully;
use crate::sketchbook::event_utils::mk_model_event;
use crate::sketchbook::model::Monotonicity;
use crate::sketchbook::JsonSerde;

use super::utils::{load_test_model, run_inference};

#[test]
/// Test inference using the test model with no changes.
fn inference_pure_model() {
    let sketch = load_test_model();
    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 32);
}

#[test]
/// Test inference using the test model with additional constraint on
/// regulation 'D -* D' being dual (instead of unspecified).
fn inference_added_dual() {
    let mut sketch = load_test_model();

    // set the dual regulation via event (and let it propagate)
    let new_sign = Monotonicity::Dual.to_json_str();
    let at_path = ["model", "regulation", "D", "D", "set_sign"];
    let event = mk_model_event(&at_path, Some(&new_sign));
    apply_event_fully(&mut sketch, &event, &at_path);

    let results = run_inference(sketch);
    assert_eq!(results.num_sat_networks, 0);
}
