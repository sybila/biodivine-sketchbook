use crate::sketchbook::bn_utils;
use crate::sketchbook::model::Monotonicity;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::properties::StatProperty;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColors, RegulationConstraint, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::BooleanNetwork;

pub fn eval_static_prop(
    static_prop: StatProperty,
    network: &BooleanNetwork,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColors, String> {
    // look into https://github.com/sybila/biodivine-lib-param-bn/blob/master/src/symbolic_async_graph/_impl_regulation_constraint.rs

    let context = graph.symbolic_context();
    // there might be some constraints already, and we only want to consider colors satisfying these too
    let initial_unit_colors = graph.mk_unit_colors();
    let unit_bdd = initial_unit_colors.as_bdd();

    // For each variable, compute Bdd that is true exactly when its update function is true.
    let update_function_is_true: Vec<Bdd> = network
        .variables()
        .map(|variable| {
            if let Some(function) = network.get_update_function(variable) {
                context.mk_fn_update_true(function)
            } else {
                context.mk_implicit_function_is_true(variable, &network.regulators(variable))
            }
        })
        .collect();

    match static_prop.get_prop_data() {
        StatPropertyType::GenericStatProp(_prop) => todo!(),
        StatPropertyType::FnInputEssential(_prop) => todo!(),
        StatPropertyType::RegulationEssential(prop) => {
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
