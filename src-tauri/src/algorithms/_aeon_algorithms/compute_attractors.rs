use crate::algorithms::_aeon_algorithms::algo_xie_beerel::xie_beerel_attractors;
use crate::algorithms::_aeon_algorithms::itgr::interleaved_transition_guided_reduction;
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, GraphColors, SymbolicAsyncGraph},
};

/// Computes terminal SCCs, and sorts colors according to how many attractors they have.
pub fn compute_attractors(graph: &SymbolicAsyncGraph) -> Vec<GraphColors> {
    // First, perform ITGR reduction.
    let (universe, active_variables) =
        interleaved_transition_guided_reduction(graph, graph.mk_unit_colored_vertices());

    let mut colors_by_num_attrs = Vec::new();
    colors_by_num_attrs.push(graph.mk_unit_colors());

    // Then run Xie-Beerel to actually detect the components, write the states to file
    xie_beerel_attractors(graph, &universe, &active_variables, |component| {
        process_component(&mut colors_by_num_attrs, &component);
    });

    colors_by_num_attrs
}

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
            println!("last");
            colors_by_num_attrs.push(intersect.clone());
        } else {
            colors_by_num_attrs[num_attrs + 1] =
                colors_by_num_attrs[num_attrs + 1].union(&intersect)
        }
        // remove the intersect colors from the original index
        colors_by_num_attrs[num_attrs] = colors_by_num_attrs[num_attrs].minus(&intersect)
    }
}
