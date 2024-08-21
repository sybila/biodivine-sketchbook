use crate::algorithms::fo_logic::parser::parse_and_minimize_fol_formula;
use crate::algorithms::fo_logic::utils::collect_unique_fol_vars;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::properties::StatProperty;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, SymbolicContext};
use biodivine_lib_param_bn::BooleanNetwork;
use std::cmp::max;
use std::collections::HashMap;

pub fn prepare_graph_for_static(
    bn: &BooleanNetwork,
    static_props: &Vec<StatProperty>,
    base_var_name: &str,
    unit: Option<(&Bdd, &SymbolicContext)>,
) -> Result<SymbolicAsyncGraph, String> {
    // todo: we now only consider generic FOL properties (other template variants might need special treatment too)

    let mut num_fol_vars: usize = 0;
    //let plain_context = SymbolicContext::new(&bn).unwrap();
    for prop in static_props {
        match prop.get_prop_data() {
            StatPropertyType::GenericStatProp(p) => {
                let formula = &p.raw_formula;
                let tree = parse_and_minimize_fol_formula(formula, base_var_name)?;
                let num_tree_vars = collect_unique_fol_vars(tree.clone()).len();
                num_fol_vars = max(num_fol_vars, num_tree_vars);
            }
            StatPropertyType::RegulationMonotonic(..)
            | StatPropertyType::RegulationEssential(..) => {}
            _ => todo!(),
        }
    }

    get_fol_extended_symbolic_graph(bn, num_fol_vars as u16, base_var_name, unit)
}

fn get_fol_extended_symbolic_graph(
    bn: &BooleanNetwork,
    num_fol_vars: u16,
    base_var_name: &str,
    unit: Option<(&Bdd, &SymbolicContext)>,
) -> Result<SymbolicAsyncGraph, String> {
    // we add all `num_fol_vars` new BDD vars as extra vars "related to" variable `base_var_name`
    let mut map_num_vars = HashMap::new();
    let base_var = bn
        .as_graph()
        .find_variable(base_var_name)
        .ok_or("Network is empty".to_string())?;
    map_num_vars.insert(base_var, num_fol_vars);
    let context = SymbolicContext::with_extra_state_variables(bn, &map_num_vars)?;

    // if we have some previous unit, lets transfer it to new context and use it, otherwise lets
    // make new full unit
    let new_unit = if let Some((unit, unit_ctx)) = unit {
        context
            .transfer_from(unit, unit_ctx)
            .ok_or("Internal error during BDD transfer from one context to another.".to_string())?
    } else {
        context.mk_constant(true)
    };

    SymbolicAsyncGraph::with_custom_context(bn, context, new_unit)
}
