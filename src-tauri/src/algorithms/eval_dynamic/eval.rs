use crate::sketchbook::properties::dynamic_props::DynPropertyType;
use crate::sketchbook::properties::DynProperty;
use biodivine_hctl_model_checker::model_checking::model_check_formula_dirty;
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
            let formula = dyn_property.processed_formula.to_string();
            let mc_results = model_check_formula_dirty(&formula, graph)?;
            Ok(mc_results.colors())
        }
        DynPropertyType::AttractorCount(_prop) => todo!(),
        DynPropertyType::ExistsFixedPoint(_prop) => todo!(),
        DynPropertyType::ExistsTrajectory(_prop) => todo!(),
        DynPropertyType::ExistsTrapSpace(_prop) => todo!(),
        DynPropertyType::HasAttractor(_prop) => todo!(),
    }
}
