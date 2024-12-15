use crate::app::event::Event;
use crate::app::state::{Consumed, SessionState};
use crate::inference::inference_results::InferenceResults;
use crate::inference::inference_solver::InferenceSolver;
use crate::inference::inference_type::InferenceType;
use crate::sketchbook::properties::{DynProperty, StatProperty};
use crate::sketchbook::Sketch;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

/// Wrapper to read the test model from JSON file.
pub fn load_test_model() -> Sketch {
    let mut json_sketch_file = File::open("../data/test_data/test_model_with_data.json").unwrap();
    let mut json_contents = String::new();
    json_sketch_file.read_to_string(&mut json_contents).unwrap();

    Sketch::from_custom_json(&json_contents).unwrap()
}

/// Wrapper to create an inference solver, run the inference on a given sketch, and return results.
pub fn run_inference(sketch: Sketch) -> InferenceResults {
    run_inference_check_statuses(sketch, None)
}

/// Wrapper to create an inference solver, run the inference on a given sketch, and return results.
///
/// Optionally, you can provide a number of expected status updates from the solver, and this function
/// asserts that solver sends exactly this number of them.
pub fn run_inference_check_statuses(
    sketch: Sketch,
    num_statuses: Option<usize>,
) -> InferenceResults {
    let (send_channel, rec_channel): (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut solver = InferenceSolver::new(send_channel);
    let results = solver.run_inference_modular(InferenceType::FullInference, sketch, true, true);

    // test cases are always valid sketches, so we just unwrap
    if let Some(expected_num) = num_statuses {
        let mut real_num = 0;
        while let Ok(_) = rec_channel.try_recv() {
            real_num += 1;
        }
        assert_eq!(real_num, expected_num);
    }
    results.unwrap()
}

/// Wrapper to apply an event, and if the result is `Consumed::Restart`, apply
/// all the subsequent sub-events.
pub fn apply_event_fully(sketch: &mut Sketch, event: &Event, at_path: &[&str]) {
    let result = sketch.perform_event(event, at_path).unwrap();

    if let Consumed::Restart(mut sub_events) = result {
        sub_events.reverse();
        for e in sub_events {
            let at_path_str: Vec<&str> = e.path[1..].iter().map(|s| s.as_str()).collect();
            let res_inner = sketch.perform_event(&e, &at_path_str).unwrap();
            // we only allow for single layer of restart events
            assert!(!matches!(res_inner, Consumed::Restart(_)))
        }
    }
}

/// Wrapper to add a given dynamic property to the model, run the inference, and return the number  
/// of satisfying candidates.
pub fn add_dyn_prop_and_infer(mut sketch: Sketch, property: DynProperty, id_str: &str) -> u128 {
    sketch
        .properties
        .add_dynamic_by_str(id_str, property)
        .unwrap();
    let results = run_inference(sketch);
    return results.num_sat_networks;
}

/// Wrapper to add a given static property to the model, run the inference, and return the number  
/// of satisfying candidates.
pub fn add_stat_prop_and_infer(mut sketch: Sketch, property: StatProperty, id_str: &str) -> u128 {
    sketch
        .properties
        .add_static_by_str(id_str, property)
        .unwrap();
    let results = run_inference(sketch);
    return results.num_sat_networks;
}
