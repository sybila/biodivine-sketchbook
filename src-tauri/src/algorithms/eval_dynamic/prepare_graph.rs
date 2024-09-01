use crate::sketchbook::properties::dynamic_props::DynPropertyType;
use crate::sketchbook::properties::DynProperty;
use biodivine_hctl_model_checker::mc_utils::collect_unique_hctl_vars;
use biodivine_hctl_model_checker::preprocessing::parser::parse_and_minimize_hctl_formula;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, SymbolicContext};
use biodivine_lib_param_bn::BooleanNetwork;
use std::cmp::max;
use std::collections::HashMap;

pub fn prepare_graph_for_dynamic(
    bn: &BooleanNetwork,
    dyn_props: &Vec<DynProperty>,
    unit: Option<(&Bdd, &SymbolicContext)>,
) -> Result<SymbolicAsyncGraph, String> {
    // todo: we now only consider generic HCTL properties (other template variants might need special treatment too)

    let mut num_hctl_vars = 0;
    let plain_context = SymbolicContext::new(bn).unwrap();
    for prop in dyn_props {
        match prop.get_prop_data() {
            DynPropertyType::GenericDynProp(p) => {
                let formula = &p.raw_formula;
                let tree = parse_and_minimize_hctl_formula(&plain_context, formula.as_str())?;
                let num_tree_vars = collect_unique_hctl_vars(tree.clone()).len();
                num_hctl_vars = max(num_hctl_vars, num_tree_vars);
            }
            _ => todo!(),
        }
    }

    get_hctl_extended_symbolic_graph(bn, num_hctl_vars as u16, unit)
}

fn get_hctl_extended_symbolic_graph(
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
