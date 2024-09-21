use crate::sketchbook::properties::DynProperty;
use crate::{
    algorithms::eval_dynamic::_attractors::sort_colors_by_attr_num,
    sketchbook::properties::dynamic_props::DynPropertyType,
};
use biodivine_hctl_model_checker::model_checking::model_check_formula_dirty;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};

/// Evaluate given dynamic property given the transition graph.
///
/// TODO: We still need to handle template properties.
pub fn eval_dyn_prop(
    dyn_prop: DynProperty,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColors, String> {
    match dyn_prop.get_prop_data() {
        DynPropertyType::GenericDynProp(dyn_property) => {
            // use HCTL model checking
            let formula = dyn_property.processed_formula.to_string();
            let mc_results = model_check_formula_dirty(&formula, graph)?;
            Ok(mc_results.colors())
        }
        DynPropertyType::AttractorCount(prop) => {
            // custom implementation (can be made more efficient if needed)
            // todo: optimize - first just compute fixed-points and get colors where N_fp <= MAX_ATTR

            // compute full attractors (on remaining colors) and get colors with correct n. of attrs
            let colors_per_num_attrs: Vec<GraphColors> = sort_colors_by_attr_num(graph);
            let mut sat_colors = graph.mk_empty_colors();
            for (num_attrs, color_set) in colors_per_num_attrs.iter().enumerate() {
                if num_attrs >= prop.minimal && num_attrs <= prop.maximal {
                    sat_colors = sat_colors.union(color_set)
                }
            }
            Ok(sat_colors)
        }
        DynPropertyType::ExistsTrapSpace(_prop) => {
            // custom implementation (can definitely be made more efficient if needed)

            // todo: get only colors where the observation is a trap space

            // todo: if needed, restrict colors to only a set where the TSs are non-percolable

            // todo: if needed, restrict colors to only a set where the TSs are minimal

            todo!()
        }
        DynPropertyType::ExistsTrajectory(_prop) => {
            // time series are not implemented yet
            todo!()
        }
        // currently, all other types of properties must be translated to HCTL generic properties
        _ => unreachable!(),
    }
}
