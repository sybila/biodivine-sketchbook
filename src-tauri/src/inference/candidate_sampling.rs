use biodivine_lib_bdd::BddPartialValuation;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};

use rand::prelude::StdRng;
use rand::SeedableRng;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};

/// Randomly select a color from the given set of colors.
/// This is a workaround that should be modified in the future.
pub fn pick_random_color(
    rng: &mut StdRng,
    graph: &SymbolicAsyncGraph,
    color_set: &GraphColors,
) -> GraphColors {
    let ctx = graph.symbolic_context();
    let random_witness = color_set.as_bdd().random_valuation(rng).unwrap();
    let mut partial_valuation = BddPartialValuation::empty();
    for var in ctx.parameter_variables() {
        partial_valuation.set_value(*var, random_witness[*var]);
    }
    let singleton_bdd = ctx
        .bdd_variable_set()
        .mk_conjunctive_clause(&partial_valuation);
    // We can use the "raw copy" function because into the new BDD, we only carried over
    // the BDD variables that encode network parameters.
    color_set.copy(singleton_bdd)
}

pub fn download_witnesses(
    path: &str,
    mut color_set: GraphColors,
    graph: &SymbolicAsyncGraph,
    witness_count: usize,
    seed: Option<u64>,
) -> Result<(), String> {
    // Prepare the archive first
    let archive_path = Path::new(path);
    // If there are some non existing dirs in path, create them.
    let prefix = archive_path.parent().unwrap();
    std::fs::create_dir_all(prefix).map_err(|e| format!("{e:?}"))?;
    // Create a zip writer for the desired archive.
    let archive = File::create(archive_path).map_err(|e| format!("{e:?}"))?;
    let mut zip_writer = ZipWriter::new(archive);

    let mut i = 0;
    let mut random_state: Option<StdRng> = seed.map(StdRng::seed_from_u64);

    // collect `num_witnesses` networks
    while i < witness_count && !color_set.is_empty() {
        // get singleton color for the witness
        let witness_color = if let Some(std_rng) = random_state.as_mut() {
            // For random networks, we need to be a bit more creative... (although, support for
            // this in lib-param-bn would be nice).
            pick_random_color(std_rng, graph, &color_set)
        } else {
            // The `SymbolicAsyncGraph::pick_singleton` should be deterministic.
            color_set.pick_singleton()
        };
        assert!(witness_color.is_singleton());

        // remove the color from the set
        color_set = color_set.minus(&witness_color);
        i += 1;

        // Write the network into the zip.
        let file_content = graph.pick_witness(&witness_color).to_string();
        zip_writer
            .start_file(format!("witness_{i}.aeon"), FileOptions::default())
            .map_err(|e| format!("{e:?}"))?;
        writeln!(zip_writer, "{file_content}").map_err(|e| format!("{e:?}"))?;
    }

    zip_writer.finish().map_err(|e| format!("{e:?}"))?;
    Ok(())
}
