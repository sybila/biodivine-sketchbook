//! Modified versions of algorithms adapted from [AEON](https://biodivine.fi.muni.cz/aeon/).
//! These algorithms can be used to compute attractor states and optimize some procedures.

/// Interleaved transition guided reduction quickly eliminates most non-attractor states.
mod itgr;
/// Xie-Beerel TSCC algorithm
mod xie_beerel;

use crate::algorithms::eval_dynamic::_attractors::itgr::interleaved_transition_guided_reduction;
use crate::algorithms::eval_dynamic::_attractors::xie_beerel::xie_beerel_attractors;
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, GraphColors, SymbolicAsyncGraph},
};

/// Compute terminal SCCs, and sort all the colors according to how many attractors they have.
/// Returns the vector, where on index i are all colors with i attractors.
pub fn sort_colors_by_attr_num<F: FnMut(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    progress_callback: &mut F,
) -> Vec<GraphColors> {
    // First, perform ITGR reduction.
    let initial = graph.mk_unit_colored_vertices();
    progress_callback(&initial, "Starting state space pre-pruning using the ITGR.");
    // TODO: Add internal progress callback into the ITGR as well
    let (universe, active_variables) = interleaved_transition_guided_reduction(graph, initial);

    let mut colors_by_num_attrs = Vec::new();
    colors_by_num_attrs.push(graph.mk_unit_colors());

    // Then run Xie-Beerel to actually detect the SCCs
    progress_callback(&universe, "Starting attractor computation with Xie-Beerel.");
    xie_beerel_attractors(
        graph,
        &universe,
        &active_variables,
        |component| {
            process_component(&mut colors_by_num_attrs, &component);
        },
        progress_callback,
    );

    colors_by_num_attrs
}

/// Process a component found by Xie-Beerel (attractor component for a subset of colors).
/// Update the `colors_by_num_attrs` so that on index i are all colors with i attractors,
/// after taking the new component into account.
fn process_component(colors_by_num_attrs: &mut Vec<GraphColors>, component: &GraphColoredVertices) {
    let component_colors = component.colors();
    let tmp_colors_by_num_attrs = colors_by_num_attrs.clone();

    for (num_attrs, color_set) in tmp_colors_by_num_attrs.into_iter().enumerate().rev() {
        // colors that had `num_attrs` before, but now we found another one
        let intersect = color_set.intersect(&component_colors);
        if intersect.is_empty() {
            continue;
        }

        // move the intersect colors one index up
        if num_attrs == colors_by_num_attrs.len() - 1 {
            colors_by_num_attrs.push(intersect.clone());
        } else {
            colors_by_num_attrs[num_attrs + 1] =
                colors_by_num_attrs[num_attrs + 1].union(&intersect)
        }
        // remove the intersect colors from the original index
        colors_by_num_attrs[num_attrs] = colors_by_num_attrs[num_attrs].minus(&intersect)
    }
}
