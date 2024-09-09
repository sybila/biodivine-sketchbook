use crate::algorithms::fo_logic::eval_wrappers::eval_formula_dirty;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::properties::StatProperty;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};

pub fn eval_static_prop(
    static_prop: StatProperty,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<GraphColors, String> {
    // there might be some constraints already, and we only want to consider colors satisfying these too
    let initial_unit_colors = graph.mk_unit_colors();

    match static_prop.get_prop_data() {
        StatPropertyType::GenericStatProp(prop) => {
            let formula = prop.processed_formula.to_string();
            let results = eval_formula_dirty(&formula, graph, base_var_name)?;
            Ok(results.colors().intersect(&initial_unit_colors))
        }
        // currently, all other types of properties must be translated to FOL generic properties
        _ => unreachable!(),
    }
}
