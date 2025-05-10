use biodivine_sketchbook::inference::candidate_sampling::download_witnesses;
use biodivine_sketchbook::inference::inference_results::InferenceResults;
use biodivine_sketchbook::inference::inference_solver::InferenceSolver;
use biodivine_sketchbook::inference::inference_type::InferenceType;
use biodivine_sketchbook::inference::results_export::export_results;
use biodivine_sketchbook::inference::sampling_data::SamplingData;
use biodivine_sketchbook::logging;
use biodivine_sketchbook::sketchbook::Sketch;

use clap::builder::PossibleValuesParser;
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
    about = "Run Boolean network inference using provided BN sketch as input."
)]
struct Arguments {
    /// Path to a file with a BN sketch in supported format (see `input-format`).
    model_path: String,

    /// Format of the input. Default is `json`, and we also support `aeon` format.
    #[clap(long, default_value = "json", value_parser = PossibleValuesParser::new(["json", "aeon"]))]
    input_format: String,

    /// Path for the zip to export results to. If not specified, only a summary is printed.
    #[clap(long)]
    results_path: Option<String>,

    /// Enable progress logging during the computation.
    #[clap(long, action)]
    logging: bool,

    /// Path for the zip to sample candidate BNs to. Number of candidates specified by `sampling-count`.
    #[clap(long, requires = "sampling_count")]
    sampling_path: Option<String>,

    /// Number of candidate BNs to sample. Path for sampling specified by `sampling-path`.
    #[clap(long, requires = "sampling_path")]
    sampling_count: Option<usize>,

    /// Seed for random generator for candidate sampling. If not specified, the sampling is deterministic.
    #[clap(long, requires = "sampling_path")]
    sampling_seed: Option<u64>,
}

/// Wrapper to create an inference solver, run the inference on a given sketch, and return results.
/// Optionally, export the results to a specified path and sample candidates.
pub fn run_inference(
    sketch: &Sketch,
    export_path: Option<String>,
    sampling_data: Option<SamplingData>,
) -> InferenceResults {
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

    // If required, export the results.
    if let Some(archive_path) = export_path {
        let finished_solver = solver
            .to_finished_solver()
            .expect("Solver did not successfully finish.");
        export_results(&archive_path, &finished_solver, sketch).expect("Error exporting results.");
    }

    // If required, sample and export candidates.
    if let Some(sampling_data) = sampling_data {
        let finished_solver = solver
            .to_finished_solver()
            .expect("Solver did not successfully finish.");
        download_witnesses(
            &sampling_data.path,
            finished_solver.sat_colors.clone(),
            &finished_solver.bn,
            sampling_data.count,
            sampling_data.seed,
        )
        .expect("Error sampling candidates.");
    }

    results
}

fn main() {
    let args = Arguments::parse();
    if !args.logging {
        // disable logging if not required
        logging::disable_logging();
    }

    // load the file contents
    let mut sketch_file =
        File::open(args.model_path.as_str()).expect("Provided file does not exist.");
    let mut file_contents = String::new();
    sketch_file
        .read_to_string(&mut file_contents)
        .expect("Error reading provided file.");

    // parse the sketch in the specified format
    let sketch = match args.input_format.as_str() {
        "json" => Sketch::from_custom_json(&file_contents).expect("Error parsing the sketch."),
        "aeon" => Sketch::from_aeon(&file_contents).expect("Error parsing the sketch."),
        _ => panic!("Unsupported input format."),
    };

    // prepare sampling data if required
    let sampling_data = if args.sampling_path.is_some() {
        if args.sampling_count.is_none() {
            panic!("Sampling path provided, but sampling count is not specified.");
        }

        Some(SamplingData {
            count: args.sampling_count.unwrap(),
            seed: args.sampling_seed,
            path: args.sampling_path.unwrap(),
        })
    } else {
        None
    };

    let inference_results = run_inference(&sketch, args.results_path, sampling_data);

    // Print the results summary
    println!();
    println!(
        "Number of candidates: {}",
        inference_results.num_sat_networks
    );
    println!("Computation time: {}ms", inference_results.comp_time);
    println!("{}", inference_results.summary_message);
}
