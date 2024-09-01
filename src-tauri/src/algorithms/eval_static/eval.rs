use crate::algorithms::fo_logic::eval_wrappers::eval_formula_dirty;
use crate::sketchbook::bn_utils;
use crate::sketchbook::model::Monotonicity;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::properties::StatProperty;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColors, RegulationConstraint, SymbolicAsyncGraph, SymbolicContext,
};
use biodivine_lib_param_bn::BooleanNetwork;

pub fn eval_static_prop(
    static_prop: StatProperty,
    network: &BooleanNetwork,
    graph: &SymbolicAsyncGraph,
    base_var_name: &str,
) -> Result<GraphColors, String> {
    // look into https://github.com/sybila/biodivine-lib-param-bn/blob/master/src/symbolic_async_graph/_impl_regulation_constraint.rs

    let context = graph.symbolic_context();
    // there might be some constraints already, and we only want to consider colors satisfying these too
    let initial_unit_colors = graph.mk_unit_colors();
    let unit_bdd = initial_unit_colors.as_bdd();

    match static_prop.get_prop_data() {
        StatPropertyType::GenericStatProp(prop) => {
            let formula = prop.processed_formula.to_string();
            let results = eval_formula_dirty(&formula, graph, base_var_name)?;
            Ok(results.colors().intersect(&initial_unit_colors))
        }
        StatPropertyType::FnInputEssential(_prop) => todo!(),
        StatPropertyType::RegulationEssential(prop) => {
            // For each variable, compute Bdd that is true exactly when its update function is true.
            let update_function_is_true: Vec<Bdd> = mk_all_updates_true(context, network);

            let input_name = prop.clone().input.unwrap();
            let target_name = prop.clone().target.unwrap();
            let input_var = network
                .as_graph()
                .find_variable(input_name.as_str())
                .unwrap();
            let target_var = network
                .as_graph()
                .find_variable(target_name.as_str())
                .unwrap();

            let fn_is_true = &update_function_is_true[target_var.to_index()];

            let observable = bn_utils::essentiality_to_bool(prop.value);
            let observability = if observable {
                RegulationConstraint::mk_observability(context, fn_is_true, input_var)
            } else {
                context.mk_constant(true)
            };
            let valid_colors = GraphColors::new(observability.and(unit_bdd), context);
            Ok(valid_colors)
        }
        StatPropertyType::RegulationEssentialContext(_prop) => todo!(),
        StatPropertyType::RegulationMonotonic(prop) => {
            // For each variable, compute Bdd that is true exactly when its update function is true.
            let update_function_is_true: Vec<Bdd> = mk_all_updates_true(context, network);

            let input_name = prop.clone().input.unwrap();
            let target_name = prop.clone().target.unwrap();
            let input_var = network
                .as_graph()
                .find_variable(input_name.as_str())
                .unwrap();
            let target_var = network
                .as_graph()
                .find_variable(target_name.as_str())
                .unwrap();

            let fn_is_true = &update_function_is_true[target_var.to_index()];

            let monotonicity = match prop.value {
                Monotonicity::Activation => {
                    RegulationConstraint::mk_activation(context, fn_is_true, input_var)
                }
                Monotonicity::Inhibition => {
                    RegulationConstraint::mk_inhibition(context, fn_is_true, input_var)
                }
                Monotonicity::Unknown => context.mk_constant(true),
                Monotonicity::Dual => unimplemented!(),
            };
            let valid_colors = GraphColors::new(monotonicity.and(unit_bdd), context);
            Ok(valid_colors)
        }
        StatPropertyType::RegulationMonotonicContext(_prop) => todo!(),
        StatPropertyType::FnInputEssentialContext(_prop) => todo!(),
        StatPropertyType::FnInputMonotonic(_prop) => todo!(),
        StatPropertyType::FnInputMonotonicContext(_prop) => todo!(),
    }
}

/// For each variable, compute Bdd that is true exactly when its update function is true.
///
/// This covers both the case when a variable has some update expression, and the case when
/// it has "implicit" (empty) update function.
fn mk_all_updates_true(context: &SymbolicContext, network: &BooleanNetwork) -> Vec<Bdd> {
    network
        .variables()
        .map(|variable| {
            if let Some(function) = network.get_update_function(variable) {
                context.mk_fn_update_true(function)
            } else {
                context.mk_implicit_function_is_true(variable, &network.regulators(variable))
            }
        })
        .collect()
}
