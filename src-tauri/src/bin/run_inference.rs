use biodivine_sketchbook::inference::inference_results::InferenceResults;
use biodivine_sketchbook::inference::inference_solver::InferenceSolver;
use biodivine_sketchbook::inference::inference_type::InferenceType;
use biodivine_sketchbook::logging;
use biodivine_sketchbook::sketchbook::Sketch;

use clap::Parser;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(
    author = "Ondřej Huvar",
    about = "Run the inference of BNs from predefined sketch."
)]
struct Arguments {
    /// Path to a file with a model in aeon sketch format.
    model_path: String,
}

/// Wrapper to create an inference solver, run the inference on a given sketch, and return results.
pub fn get_inference_results(sketch: &Sketch) -> InferenceResults {
    let (send_channel, rec_channel): (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut solver = InferenceSolver::new(send_channel);
    let results =
        solver.run_inference_modular(InferenceType::FullInference, sketch.clone(), true, true);
    loop {
        if rec_channel.try_recv().is_err() {
            break;
        }
    }
    results.expect("The computation was not successful.")
}

fn main() {
    let args = Arguments::parse();
    // we disable logging since it would only overflow the output
    logging::disable_logging();

    // load the sketch
    let mut sketch_file =
        File::open(args.model_path.as_str()).expect("Provided file does not exist.");
    let mut file_contents = String::new();
    sketch_file
        .read_to_string(&mut file_contents)
        .expect("Error reading provided file.");
    let sketch = Sketch::from_aeon(&file_contents).expect("Error parsing the sketch.");

    let inference_results = get_inference_results(&sketch);
    println!(
        "Number of candidates: {}",
        inference_results.num_sat_networks
    );
    println!("Computation time: {}ms", inference_results.comp_time);
    println!("{}", inference_results.summary_message);
}
