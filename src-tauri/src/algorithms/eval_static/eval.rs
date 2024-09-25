use crate::algorithms::eval_static::processed_props::ProcessedStatProp;
use crate::algorithms::fo_logic::eval_wrappers::eval_formula_dirty;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};

/// Evaluate given static property.
///
/// Currently, we assume that all types of template properties must be already translated
/// to FOL generic properties.
pub fn eval_static_prop(
    static_prop: ProcessedStatProp,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<GraphColors, String> {
    // there might be some constraints already, and we only want to consider colors satisfying these too
    let initial_unit_colors = graph.mk_unit_colors();

    let formula = static_prop.formula.to_string();
    let results = eval_formula_dirty(&formula, graph, base_var_name)?;
    Ok(results.colors().intersect(&initial_unit_colors))
}
