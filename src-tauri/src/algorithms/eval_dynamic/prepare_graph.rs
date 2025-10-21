use crate::algorithms::eval_dynamic::processed_props::ProcessedDynProp;

use biodivine_hctl_model_checker::mc_utils::collect_unique_hctl_vars;
use biodivine_hctl_model_checker::preprocessing::parser::parse_and_minimize_extended_formula;
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
/// variables to handle all variables in HCTL formulas. Essentially, we need to count the
/// maximal num of HCTL vars per formula (as the variables are normalized and same vars are
/// reused throughout all formulas). We always have at least one set of symbolic variables,
/// as they can be used when computing trap spaces.
///
/// Note that some cases like trap spaces need different kind of symbolic context and
/// graph, but this context is always the same and is easily handled during evaluation.
///
/// Arg `unit` is optional unit BDD with potentially slightly different symbolic context (it
/// can have different UNUSED symbolic variables, but has the same bn vars and colors).
/// This can be useful for example when transfering between HCTL model checking and FOL
/// checking contexts.
pub fn prepare_graph_for_dynamic_hctl(
    bn: &BooleanNetwork,
    dyn_props: &Vec<ProcessedDynProp>,
    unit: Option<(&Bdd, &SymbolicContext)>,
) -> Result<SymbolicAsyncGraph, String> {
    let mut num_hctl_vars = 0;
    // context needed to process formulas
    let plain_context = SymbolicContext::new(bn).unwrap();

    for prop in dyn_props {
        let num_vars_prop = count_num_hctl_vars_single(prop, &plain_context)?;
        num_hctl_vars = max(num_hctl_vars, num_vars_prop)
    }

    // always add at least one set of symbolic variables, as they can be used when computing trap spaces
    num_hctl_vars = max(1, num_hctl_vars);

    get_hctl_extended_symbolic_graph(bn, num_hctl_vars as u16, unit)
}

/// Count the number of HCTL variables needed for evaluation of a single property.
/// If the property contains some sub-properties, compute recursively as maximum.
pub fn count_num_hctl_vars_single(
    prop: &ProcessedDynProp,
    context: &SymbolicContext,
) -> Result<usize, String> {
    let num_hctl_vars = match &prop {
        ProcessedDynProp::ProcessedHctlFormula(p) => {
            // process the main formula
            let tree = parse_and_minimize_extended_formula(context, &p.formula)?;
            let mut num_vars = collect_unique_hctl_vars(tree.clone()).len();

            // and also process all potential sub-properties (these will be handled individually too)
            for sub_prop in p.sub_properties.iter() {
                let num_sub_prop_vars = count_num_hctl_vars_single(sub_prop, context)?;
                num_vars = max(num_vars, num_sub_prop_vars);
            }
            num_vars
        }
        // no need for any additional variables for an attractor count property
        ProcessedDynProp::ProcessedAttrCount(..) => 0,
        // no need for any additional variables for a simple trajectory property
        ProcessedDynProp::ProcessedSimpleTrajectory(..) => 0,
        // no need for any additional variables for a fixed point property
        ProcessedDynProp::ProcessedFixedPoint(..) => 0,
        // this one is handled entirely later during evaluation
        ProcessedDynProp::ProcessedTrapSpace(..) => 0,
        // no need for any additional variables for an observation sub-property
        ProcessedDynProp::ProcessedObservation(..) => 0,
    };
    Ok(num_hctl_vars)
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
    let context = SymbolicSpaceContext::new(bn);
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::vec;

    use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, SymbolicContext};
    use biodivine_lib_param_bn::BooleanNetwork;

    use crate::algorithms::eval_dynamic::prepare_graph::{
        get_hctl_extended_symbolic_graph, prepare_graph_for_dynamic_hctl,
    };
    use crate::algorithms::eval_dynamic::processed_props::ProcessedDynProp;

    #[test]
    /// Test automatic generation of symbolic context for HCTL properties.
    fn test_prepare_context_hctl() {
        let bn = BooleanNetwork::try_from("a -> a").unwrap();
        let canonical_context = SymbolicContext::new(&bn).unwrap();
        let canonical_unit = canonical_context.mk_constant(true);

        // we'll use 1 HCTL variable
        let var_a = bn.as_graph().find_variable("a").unwrap();
        let hctl_vars_map = HashMap::from([(var_a, 1)]);
        let hctl_context =
            SymbolicContext::with_extra_state_variables(&bn, &hctl_vars_map).unwrap();

        // test manual HCTL graph creation
        let hctl_unit = hctl_context.mk_constant(true);
        let graph_hctl_expected =
            SymbolicAsyncGraph::with_custom_context(&bn, hctl_context.clone(), hctl_unit).unwrap();
        // a) from scratch
        let graph_hctl = get_hctl_extended_symbolic_graph(&bn, 1, None).unwrap();
        assert_eq!(graph_hctl_expected.unit_colors(), graph_hctl.unit_colors());
        // b) converting from canonical unit BDD
        let graph_hctl =
            get_hctl_extended_symbolic_graph(&bn, 1, Some((&canonical_unit, &canonical_context)))
                .unwrap();
        assert_eq!(graph_hctl_expected.unit_colors(), graph_hctl.unit_colors());

        // test deriving HCTL context automatically from property
        let property_list = vec![ProcessedDynProp::mk_hctl(
            "doesntmatter",
            "3{x}: AX {x}",
            Vec::new(),
        )];
        let graph_hctl = prepare_graph_for_dynamic_hctl(&bn, &property_list, None).unwrap();
        assert_eq!(graph_hctl_expected.unit_colors(), graph_hctl.unit_colors());
    }
}
