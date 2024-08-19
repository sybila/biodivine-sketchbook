use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, SymbolicContext};
use biodivine_lib_param_bn::BooleanNetwork;
use std::collections::HashMap;

pub fn get_hctl_extended_symbolic_graph(
    bn: &BooleanNetwork,
    num_hctl_vars: u16,
    unit: Option<(&Bdd, &SymbolicContext)>,
) -> Result<SymbolicAsyncGraph, String> {
    // for each BN var, `num_hctl_vars` new BDD vars must be created
    let mut map_num_vars = HashMap::new();
    for bn_var in bn.variables() {
        map_num_vars.insert(bn_var, num_hctl_vars);
    }
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

pub fn get_fol_extended_symbolic_graph(
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
