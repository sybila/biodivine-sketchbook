use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};

use crate::sketchbook::Sketch;

use super::{inference_results::InferenceResults, inference_solver::FinishedInferenceSolver};

/// Export archive with complete results to the given path.
/// The results archive include:
/// - a summary report (basically information tracked by the `InferenceResults` struct)
/// - original sketch in JSON format for replicability in SketchBook
/// - BDD with satisfying colors
/// - a PSBN model derived from the sketch (in aeon format) that can be used as a context for the BDD
pub fn export_results(
    path: &str,
    finished_solver: &FinishedInferenceSolver,
    original_sketch: &Sketch,
) -> Result<(), String> {
    // Prepare the archive first
    let archive_path = Path::new(path);
    // If there are some non existing dirs in path, create them.
    let prefix = archive_path.parent().unwrap();
    std::fs::create_dir_all(prefix).map_err(|e| format!("{e:?}"))?;
    // Create a zip writer for the desired archive.
    let archive = File::create(archive_path).map_err(|e| format!("{e:?}"))?;
    let mut zip_writer = ZipWriter::new(archive);

    // write the sketch model
    let sketch_json = original_sketch.to_custom_json();
    write_to_zip("original_sketch.json", &mut zip_writer, sketch_json)?;

    // write the color BDD
    let color_bdd = finished_solver.sat_colors.as_bdd();
    let color_bdd_str = color_bdd.to_string();
    write_to_zip("color_bdd.bdd", &mut zip_writer, color_bdd_str)?;

    // write the BN model
    let bn_model_aeon = finished_solver.bn.to_string();
    write_to_zip("derived_model.aeon", &mut zip_writer, bn_model_aeon)?;

    // write the report
    let formatted_report = format_inference_results(&finished_solver.results);
    write_to_zip("report.txt", &mut zip_writer, formatted_report)?;

    zip_writer.finish().map_err(|e| format!("{e:?}"))?;
    Ok(())
}

/// Helper function to write string into a zip archive file.
fn write_to_zip(
    file_name: &str,
    zip_writer: &mut ZipWriter<File>,
    file_content: String,
) -> Result<(), String> {
    zip_writer
        .start_file(file_name, FileOptions::default())
        .map_err(|e| format!("{e:?}"))?;
    write!(zip_writer, "{file_content}").map_err(|e| format!("{e:?}"))?;
    Ok(())
}

fn format_inference_results(results: &InferenceResults) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "Number of satisfying candidates: {}\n",
        results.num_sat_networks
    ));
    output.push_str(&format!(
        "Computation time: {} milliseconds\n\n",
        results.comp_time
    ));
    output.push_str("--------------\n");
    output.push_str("Extended summary:\n");
    output.push_str("--------------\n");
    output.push_str(&format!("{}\n", results.summary_message));

    output.push_str("--------------\n");
    output.push_str("Detailed progress report:\n");
    output.push_str("--------------\n");

    for report in &results.progress_statuses {
        let report_line = match report.num_candidates {
            Some(candidates) => format!(
                "> {}ms: {} ({} candidates)\n",
                report.comp_time, report.message, candidates
            ),
            None => format!("> {}ms: {}\n", report.comp_time, report.message),
        };
        output.push_str(&report_line);
    }

    output
}
