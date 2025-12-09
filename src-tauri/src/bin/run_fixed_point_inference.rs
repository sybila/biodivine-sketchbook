use biodivine_sketchbook::inference::inference_results::InferenceResults;
use biodivine_sketchbook::inference::inference_solver::InferenceSolver;
use biodivine_sketchbook::inference::inference_type::InferenceType;
use biodivine_sketchbook::inference::results_export::export_results;
use biodivine_sketchbook::logging;
use biodivine_sketchbook::sketchbook::Sketch;

use biodivine_sketchbook::sketchbook::properties::DynProperty;
use clap::Parser;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(
    author = "Ondřej Huvar",
    version,
    about = "Run simplified BN inference using provided PSBN and fixed point-data."
)]
struct Arguments {
    /// Path to a file with PSBN model in AEON format.
    psbn_path: String,

    /// Path to a file with fixed-point dataset in CSV format.
    csv_path: String,

    /// Export path for the zip with the results.
    results_path: String,
}

/// Create an inference solver, run the inference on a given sketch, and return results.
/// In the end, export the symbolic results to a specified path.
pub fn run_inference(sketch: &Sketch, export_path: &str) -> InferenceResults {
    // Create a new inference solver, and set up communication channels.
    let (send_channel, rec_channel): (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut solver = InferenceSolver::new(send_channel);

    // Run the inference and discard all progress messages intended to be sent to GUI.
    let results = solver
        .run_inference_modular(InferenceType::FullInference, sketch.clone(), true, true)
        .expect("The computation was not successful.");
    loop {
        if rec_channel.try_recv().is_err() {
            break;
        }
    }

    // Export the results.
    let finished_solver = solver
        .to_finished_solver()
        .expect("Solver did not successfully finish.");
    export_results(export_path, &finished_solver, sketch).expect("Error exporting results.");

    results
}

fn main() {
    logging::disable_logging();
    let args = Arguments::parse();
    println!("Input model: `{}`", args.psbn_path,);
    println!("Input data: `{}`", args.csv_path,);
    println!("Running inference with fixed-point specification...");

    // Load the aeon file contents and parse PSBN into an sketch instance
    let mut sketch_file =
        File::open(args.psbn_path.as_str()).expect("Provided aeon file does not exist.");
    let mut file_contents = String::new();
    sketch_file
        .read_to_string(&mut file_contents)
        .expect("Error reading provided aeon file.");
    let mut sketch = Sketch::from_aeon(&file_contents).expect("Error parsing the sketch.");

    // Process the csv data and add it to the sketch
    let data_id_str = "fps_data";
    sketch
        .load_dataset(data_id_str, args.csv_path.as_str())
        .expect("Error processing provided csv data file.");

    // Add fixed-point property referencing all observations of the dataset
    let prop_id_str = "fps_property";
    let dataset_id = sketch.observations.get_dataset_id(data_id_str).unwrap();
    let property = DynProperty::mk_fixed_point(prop_id_str, Some(dataset_id), None);
    sketch
        .properties
        .add_dynamic_by_str(prop_id_str, property)
        .unwrap();

    // Run the actual inference procedure (including export)
    let inference_results = run_inference(&sketch, &args.results_path);

    // Print simple results summary
    println!(
        "Number of candidates: {}",
        inference_results.num_sat_networks
    );
    println!("Computation time: {}ms", inference_results.comp_time);
    println!("Exported results to {}\n", &args.results_path);
}
