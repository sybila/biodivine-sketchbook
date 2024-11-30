use crate::algorithms::eval_static::processed_props::ProcessedStatProp;
use crate::algorithms::fo_logic::parser::parse_and_minimize_fol_formula;
use crate::algorithms::fo_logic::utils::collect_unique_fol_vars;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, SymbolicContext};
use biodivine_lib_param_bn::BooleanNetwork;
use std::cmp::max;
use std::collections::HashMap;

/// Prepare the symbolic context and generate the symbolic transition graph for
/// evaluation of the static properties.
///
/// Since all the static properties are encoded as FOL properties, we just need to
/// handle this one case. This means we need to prepare symbolic variables to cover
/// all variables in FOL formulas.
///
/// Arg `unit` is optional unit BDD with potentially different symbolic context (can
/// have different symbolic variables, but has the same bn vars and colors).
pub fn prepare_graph_for_static_fol(
    bn: &BooleanNetwork,
    static_props: &Vec<ProcessedStatProp>,
    base_var_name: &str,
    unit: Option<(&Bdd, &SymbolicContext)>,
) -> Result<SymbolicAsyncGraph, String> {
    // we now assume all properties are already encoded into generic FOL properties

    let mut num_fol_vars: usize = 0;
    //let plain_context = SymbolicContext::new(&bn).unwrap();
    for prop in static_props {
        let formula = &prop.formula;
        let tree = parse_and_minimize_fol_formula(formula, base_var_name)?;
        let num_tree_vars = collect_unique_fol_vars(&tree).len();
        num_fol_vars = max(num_fol_vars, num_tree_vars);
    }

    get_fol_extended_symbolic_graph(bn, num_fol_vars as u16, base_var_name, unit)
}

/// Prepare the symbolic context and generate the symbolic transition graph for
/// evaluation of FOL formulas. This means we need to prepare symbolic variables to
/// cover all variables in FOL formulas.
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::vec;

    use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, SymbolicContext};
    use biodivine_lib_param_bn::BooleanNetwork;

    use crate::algorithms::eval_static::prepare_graph::{
        get_fol_extended_symbolic_graph, prepare_graph_for_static_fol,
    };
    use crate::algorithms::eval_static::processed_props::ProcessedStatProp;

    #[test]
    /// Test automatic generation of symbolic context for FOL properties.
    fn test_prepare_context_fol() {
        let bn = BooleanNetwork::try_from("a -> a").unwrap();
        let canonical_context = SymbolicContext::new(&bn).unwrap();
        let canonical_unit = canonical_context.mk_constant(true);

        // we'll use 1 FOL variable
        let var_a = bn.as_graph().find_variable("a").unwrap();
        let fol_vars_map = HashMap::from([(var_a, 1)]);
        let fol_context = SymbolicContext::with_extra_state_variables(&bn, &fol_vars_map).unwrap();

        // test manual FOL graph creation
        let fol_unit = fol_context.mk_constant(true);
        let graph_fol_expected =
            SymbolicAsyncGraph::with_custom_context(&bn, fol_context.clone(), fol_unit).unwrap();
        // a) from scratch
        let graph_fol = get_fol_extended_symbolic_graph(&bn, 1, "a", None).unwrap();
        assert_eq!(graph_fol_expected.unit_colors(), graph_fol.unit_colors());
        // b) converting from canonical unit BDD
        let graph_fol = get_fol_extended_symbolic_graph(
            &bn,
            1,
            "a",
            Some((&canonical_unit, &canonical_context)),
        )
        .unwrap();
        assert_eq!(graph_fol_expected.unit_colors(), graph_fol.unit_colors());

        // test FOL graph creation automatically from property
        let fol_prop = ProcessedStatProp::mk_fol("doesntmatter", "3 x: true");
        let property_list = vec![fol_prop];
        let graph_fol = prepare_graph_for_static_fol(&bn, &property_list, "a", None).unwrap();
        assert_eq!(graph_fol_expected.unit_colors(), graph_fol.unit_colors());
    }
}
