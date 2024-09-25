use crate::algorithms::eval_dynamic::processed_props::ProcessedDynProp;

use biodivine_hctl_model_checker::mc_utils::collect_unique_hctl_vars;
use biodivine_hctl_model_checker::preprocessing::parser::parse_and_minimize_hctl_formula;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, SymbolicAsyncGraph, SymbolicContext,
};
use biodivine_lib_param_bn::trap_spaces::SymbolicSpaceContext;
use biodivine_lib_param_bn::BooleanNetwork;

use std::cmp::max;
use std::collections::HashMap;

/// Prepare the symbolic context and generate the symbolic transition graph for
/// evaluation of the dynamic properties.
///
/// Most of the properties are encoded as HCTL formulas. Therefore we just prepare symbolic
/// variables to handle all variables in HCTL formulas. We always have at least one set
/// of symbolic variables, as they can be used when computing trap spaces.
///
/// Note that some cases like trap spaces need different kind of symbolic context and
/// graph, but this context is always the same and is easily handled during evaluation.
///
/// Arg `unit` is optional unit BDD with potentially different symbolic context (can
/// have different symbolic variables, but has the same bn vars and colors).
pub fn prepare_graph_for_dynamic_hctl(
    bn: &BooleanNetwork,
    dyn_props: &Vec<ProcessedDynProp>,
    unit: Option<(&Bdd, &SymbolicContext)>,
) -> Result<SymbolicAsyncGraph, String> {
    let mut num_hctl_vars = 0;
    let plain_context = SymbolicContext::new(bn).unwrap();

    for prop in dyn_props {
        match &prop {
            ProcessedDynProp::ProcessedHctlFormula(p) => {
                let tree = parse_and_minimize_hctl_formula(&plain_context, &p.formula)?;
                let num_tree_vars = collect_unique_hctl_vars(tree.clone()).len();
                num_hctl_vars = max(num_hctl_vars, num_tree_vars);
            }
            // no need for any additional variables for attractor count property
            ProcessedDynProp::ProcessedAttrCount(..) => {}
            // this one is handled entirely later during evaluation
            ProcessedDynProp::ProcessedTrapSpace(..) => {}
        }
    }

    // always add at least one set of symbolic variables, as they can be used when computing trap spaces
    num_hctl_vars = max(1, num_hctl_vars);

    get_hctl_extended_symbolic_graph(bn, num_hctl_vars as u16, unit)
}

/// Prepare the symbolic context and generate the symbolic transition graph for
/// evaluation of HCTL formulas. This means we need to prepare symbolic variables to
/// cover all variables in these HCTL formulas.
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
    let new_unit_bdd = if let Some((unit_bdd, unit_ctx)) = unit {
        context
            .transfer_from(unit_bdd, unit_ctx)
            .ok_or("Internal error during BDD transfer from one context to another.".to_string())?
    } else {
        context.mk_constant(true)
    };

    SymbolicAsyncGraph::with_custom_context(bn, context, new_unit_bdd)
}

/// Prepare the symbolic context and generate the symbolic transition graph for
/// computation of trap spaces.
pub fn get_ts_extended_symbolic_graph(
    bn: &BooleanNetwork,
    unit: Option<(&Bdd, &SymbolicContext)>,
) -> Result<(SymbolicSpaceContext, SymbolicAsyncGraph), String> {
    let context: SymbolicSpaceContext = SymbolicSpaceContext::new(bn);
    let graph = SymbolicAsyncGraph::with_space_context(bn, &context)?;

    // if we have some previous unit set restriction, lets transfer it to new context and use it
    let new_unit_bdd = if let Some((unit_bdd, unit_ctx)) = unit {
        context
            .inner_context()
            .transfer_from(unit_bdd, unit_ctx)
            .ok_or("Internal error during BDD transfer from one context to another.".to_string())?
    } else {
        context.inner_context().mk_constant(true)
    };

    let new_unit_set = GraphColoredVertices::new(new_unit_bdd, context.inner_context());
    Ok((context, graph.restrict(&new_unit_set)))
}
