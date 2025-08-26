use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;

use crate::algorithms::eval_static::encode::{
    encode_regulation_essentiality, encode_regulation_monotonicity,
};
use crate::algorithms::eval_static::eval::eval_static_prop;
use crate::algorithms::eval_static::prepare_graph::prepare_graph_for_static_fol;
use crate::algorithms::eval_static::processed_props::ProcessedStatProp;
use crate::algorithms::fo_logic::parser::parse_and_minimize_fol_formula;
use crate::inference::inference_solver::FinishedInferenceSolver;
use crate::sketchbook::model::{Essentiality, ModelState, Monotonicity};
use crate::sketchbook::Sketch;

/// Construct refined regulatory network using the set of admissible candidates.
/// Return `ModelState` struct with the refined network (and empty functions).
/// This structure utilizes the same layout as the original model for convenience.
///
/// The original input influence graph is used as a starting point, and the regulation
/// properties are refined using the set of admissible candidates. For example, if
/// originally `unspecified` regulation is found to be `activating` and `essential`
/// among all the admissible candidate networks, it is refined as that.
pub fn refine_regulatory_network(
    finished_solver: &FinishedInferenceSolver,
    original_sketch: &Sketch,
) -> Result<ModelState, String> {
    let orig_model = &original_sketch.model;
    // Inferred colors from the inference (in canonical symbolic context)
    let admissible_colors = &finished_solver.sat_colors;
    let orig_bn = &finished_solver.bn;
    let orig_graph = &finished_solver.graph;

    // Start with a model with the same variables and layout
    let variables = orig_model
        .variables()
        .map(|(v_id, v)| (v_id.as_str(), v.get_name()))
        .collect();
    let mut refined_model = ModelState::new_from_vars(variables)?;
    let orig_layout = orig_model.get_default_layout();
    refined_model.update_default_layout(orig_layout.clone())?;

    // Iterate over all original regulations
    for orig_regulation in orig_model.regulations() {
        // Prepare the regulation to potentially be refined
        let mut refined_reg = orig_regulation.clone();
        let regulator = refined_reg.get_regulator().clone();
        let target = refined_reg.get_target().clone();

        // If the regulation did not have fixed monotonicity, check if it can be refined.
        if *refined_reg.get_sign() == Monotonicity::Unknown {
            // Check if all admissible candidates satisfy some version of monotonicity
            for monotonicity in [
                Monotonicity::Activation,
                Monotonicity::Inhibition,
                Monotonicity::Dual,
            ] {
                // Encode the regulation property into FOL formula
                let formula = encode_regulation_monotonicity(
                    regulator.as_str(),
                    target.as_str(),
                    monotonicity,
                    orig_bn,
                );

                if check_if_universally_sat(&formula, admissible_colors, orig_bn, orig_graph)? {
                    refined_reg.swap_sign(monotonicity);
                    break; // no need to iterate further, the monotonicity options are non-overlapping
                }
            }
        }

        // If the regulation did not have fixed essentiality, check if it can be refined.
        if *refined_reg.get_essentiality() == Essentiality::Unknown {
            // Check if all admissible candidates satisfy some version of essentiality
            for essentiality in [Essentiality::True, Essentiality::False] {
                // Encode the regulation property into FOL formula
                let formula = encode_regulation_essentiality(
                    regulator.as_str(),
                    target.as_str(),
                    essentiality,
                    orig_bn,
                );

                // Check if all inferred candidate colors satisfy the property
                if check_if_universally_sat(&formula, admissible_colors, orig_bn, orig_graph)? {
                    refined_reg.swap_essentiality(essentiality);
                    break; // no need to iterate further, the essentiality options are non-overlapping
                }
            }
        }

        // TODO: In future, we will check for more update function properties

        refined_model.add_regulation_raw(refined_reg)?;
    }

    Ok(refined_model)
}

/// Validate whether given FOL `formula` is satisfied across all colors in the
/// `admissible_colors` set. The `orig_bn` is the original PSBN that was used
/// as an inference input, and `orig_graph` is its symbolic graph (with canonical
/// symbolic context).
pub fn check_if_universally_sat(
    formula: &str,
    admissible_colors: &GraphColors,
    orig_bn: &BooleanNetwork,
    orig_graph: &SymbolicAsyncGraph,
) -> Result<bool, String> {
    // Select a BN variable (can be random) that will be used as a base during
    // symbolic encoding of extra FOL vars in BDDs.
    let base_var = orig_bn.variables().collect::<Vec<_>>()[0];
    let base_var_name = orig_bn.as_graph().get_variable_name(base_var).clone();

    // Process the FOL string property into proper `ProcessedStatProp` instance
    let id = "placeholder_prop_id";
    let parsed_formula = parse_and_minimize_fol_formula(formula, &base_var_name)?;
    let prepared_prop = ProcessedStatProp::mk_fol(id, parsed_formula);

    // Prepare the symbolic context for evaluation of the FOL formula (and restrict the
    // unit BDD to only the admissible colors)
    let unit = Some((admissible_colors.as_bdd(), orig_graph.symbolic_context()));
    let extended_graph =
        prepare_graph_for_static_fol(orig_bn, &vec![prepared_prop.clone()], &base_var_name, unit)?;
    // Evaluate the property
    let sat_colors = eval_static_prop(&prepared_prop, &extended_graph, &base_var_name)?;

    // To compare the set of computed sat colors with the original set of all admissible colors, we
    // need to make sure they use the same symbolic context. We simply transfer the computed set into
    // the canonical context (since the previously added extra symbolic vars are no longer needed).
    let current_context = extended_graph.symbolic_context();
    let pure_context = current_context.as_canonical_context();
    let pure_sat_bdd = pure_context
        .transfer_from(sat_colors.as_bdd(), current_context)
        .unwrap();
    let pure_sat_colors = GraphColors::new(pure_sat_bdd.clone(), &pure_context);

    // Check if all the admissible colors are included in the computed sat colors
    Ok(admissible_colors.minus(&pure_sat_colors).is_empty())
}
