use crate::sketchbook::ids::{PerturbationId, VarId};
use crate::sketchbook::perturbations::PerturbationManager;
use biodivine_lib_bdd::{Bdd, BddVariable};
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColors, SymbolicAsyncGraph, SymbolicContext,
};
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate, ParameterId, VariableId};

use std::collections::{BTreeMap, HashMap};

/// Binary selector code assigned to wild type or to a specific sketch perturbation.
pub type SelectorCode = u32;
const WILD_TYPE_CODE: SelectorCode = 0;

/// Compiled encoding of sketch perturbations for dynamic property evaluation.
///
/// Contains a selector-multiplexed Boolean network and metadata for restricting or
/// projecting selector colors during inference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BnWithPerturbations {
    /// Wild-type BN plus selector-only parameters and multiplexed update functions.
    pub bn: BooleanNetwork,
    /// Zero-arity selector parameters (`pert_sel_0`, `pert_sel_1`, ...).
    /// One perturbation is represented by a valuation of these parameters.
    pub selector_params: Vec<ParameterId>,
    /// Selector code for each sketch perturbation ID.
    pub perturbation_codes: HashMap<PerturbationId, SelectorCode>,
}

impl BnWithPerturbations {
    pub fn new(
        perturbations: &PerturbationManager,
        bn: &BooleanNetwork,
    ) -> Result<BnWithPerturbations, String> {
        // If there are no perturbations, return the wild-type BN unchanged.
        if perturbations.num_perturbations() == 0 {
            return Ok(BnWithPerturbations {
                bn: bn.clone(),
                selector_params: Vec::new(),
                perturbation_codes: HashMap::new(),
            });
        }

        let mut perturbation_ids: Vec<PerturbationId> = perturbations
            .perturbations_iter()
            .map(|(id, _)| id.clone())
            .collect();
        perturbation_ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        // Prepare the selector parameters and perturbation codes.
        let num_choices = 1 + perturbation_ids.len();
        let num_selector_bits = selector_bit_count(num_choices);
        let mut parameterized_bn = bn.clone();
        let mut selector_params = Vec::with_capacity(num_selector_bits);

        for bit_index in 0..num_selector_bits {
            let param_name = format!("pert_sel_{bit_index}");
            let param_id = parameterized_bn
                .add_parameter(param_name.as_str(), 0)
                .map_err(|e| format!("Failed adding selector parameter `{param_name}`: {e}."))?;
            selector_params.push(param_id);
        }

        let mut perturbation_codes = HashMap::new();
        for (index, perturbation_id) in perturbation_ids.iter().enumerate() {
            perturbation_codes.insert(perturbation_id.clone(), (index + 1) as SelectorCode);
        }

        // Map each affected BN variable to perturbations that force its value.
        let variable_perturbations =
            collect_variable_perturbation_map(perturbations, bn, &perturbation_codes)?;
        // Rewrite the update functions of the affected variables into selector multiplexers.
        for (variable, perturbation_ids) in variable_perturbations {
            let original_update = parameterized_bn
                .get_update_function(variable)
                .clone()
                .ok_or_else(|| {
                    // This is unreachable in inference, but kept for completeness.
                    format!(
                        "Variable `{}` has no update function, cannot be perturbed.",
                        parameterized_bn.get_variable_name(variable)
                    )
                })?;
            let multiplexed_update = multiplex_update_for_variable(
                &parameterized_bn,
                &selector_params,
                &perturbation_codes,
                perturbations,
                variable,
                &perturbation_ids,
                &original_update,
            )?;
            parameterized_bn
                .set_update_function(variable, Some(multiplexed_update))
                .map_err(|e| {
                    format!(
                        "Failed setting multiplexed update for `{}`: {e}.",
                        parameterized_bn.get_variable_name(variable)
                    )
                })?;
        }

        Ok(BnWithPerturbations {
            bn: parameterized_bn,
            selector_params,
            perturbation_codes,
        })
    }

    /// Resolve the selector code for a property's applied perturbation reference.
    pub fn code_for_perturbation(
        &self,
        perturbation_id: &PerturbationId,
    ) -> Result<SelectorCode, String> {
        self.perturbation_codes
            .get(perturbation_id)
            .copied()
            .ok_or_else(|| {
                format!("Referenced perturbation `{perturbation_id}` has no selector code.")
            })
    }

    pub fn code_for_wild_type(&self) -> SelectorCode {
        WILD_TYPE_CODE
    }

    /// Resolve the selector code for a dynamic property's optional perturbation.
    pub fn code_for_applied_perturbation(
        &self,
        applied_perturbation: Option<&PerturbationId>,
    ) -> Result<SelectorCode, String> {
        match applied_perturbation {
            None => Ok(self.code_for_wild_type()),
            Some(id) => self.code_for_perturbation(id),
        }
    }

    /// Number of perturbations defined in the sketch (excluding wild type).
    pub fn num_perturbations(&self) -> usize {
        self.perturbation_codes.len()
    }

    /// Get the BDD variables representing the perturbation selector parameters.
    fn perturbation_selector_bdd_vars(&self, ctx: &SymbolicContext) -> Vec<BddVariable> {
        let mut variables = Vec::with_capacity(self.selector_params.len());
        for param in &self.selector_params {
            variables.extend(
                ctx.get_explicit_function_table(*param)
                    .symbolic_variables()
                    .iter()
                    .copied(),
            );
        }
        variables
    }

    /// Build a BDD that is satisfied exactly by the given perturbation selector code assignment.
    ///
    /// This can be used to restrict the graph to a single perturbation, or to project
    /// out the perturbation selector parameters.
    pub fn perturbation_selector_bdd(&self, ctx: &SymbolicContext, code: SelectorCode) -> Bdd {
        if self.selector_params.is_empty() {
            return ctx.mk_constant(code == 0);
        }

        let mut result = ctx.mk_constant(true);
        for (bit_index, param) in self.selector_params.iter().enumerate() {
            let bit_is_set = (code >> bit_index) & 1 != 0;
            let param_bdd = ctx.mk_uninterpreted_function_is_true(*param, &[]);
            let bit_bdd = if bit_is_set {
                param_bdd
            } else {
                param_bdd.not()
            };
            result = result.and(&bit_bdd);
        }
        result
    }

    /// Restrict the unit colors of `graph` to a single perturbation (selector variable valuations).
    pub fn restrict_graph_to_perturbation(
        &self,
        graph: &SymbolicAsyncGraph,
        code: SelectorCode,
    ) -> SymbolicAsyncGraph {
        let ctx = graph.symbolic_context();
        let selector_bdd = self.perturbation_selector_bdd(ctx, code);
        let selector_colors = graph.mk_unit_colors().copy(selector_bdd);
        let restricted_unit = graph
            .unit_colored_vertices()
            .intersect_colors(&selector_colors);
        graph.restrict(&restricted_unit)
    }

    /// Existentially project all perturbation selector parameter colors from `colors`.
    pub fn project_perturbation_selector_colors(
        &self,
        ctx: &SymbolicContext,
        colors: &GraphColors,
    ) -> GraphColors {
        // If there are no perturbation selector parameters, return the colors unchanged.
        if self.selector_params.is_empty() {
            return colors.clone();
        }

        let selector_bdd_vars = self.perturbation_selector_bdd_vars(ctx);
        let projected_bdd = colors.as_bdd().exists(&selector_bdd_vars);
        GraphColors::new(projected_bdd, ctx)
    }
}

/// Calculate the number of bits needed to represent the selector code for a given number
/// of perturbations.
fn selector_bit_count(num_choices: usize) -> usize {
    if num_choices <= 1 {
        0
    } else {
        (usize::BITS - (num_choices - 1).leading_zeros()) as usize
    }
}

/// For each BN variable, list perturbation IDs that set its value (sorted by perturbation selector code).
fn collect_variable_perturbation_map(
    perturbations: &PerturbationManager,
    bn: &BooleanNetwork,
    perturbation_codes: &HashMap<PerturbationId, SelectorCode>,
) -> Result<BTreeMap<VariableId, Vec<PerturbationId>>, String> {
    let mut variable_perturbations: BTreeMap<VariableId, Vec<PerturbationId>> = BTreeMap::new();

    for (perturbation_id, perturbation) in perturbations.perturbations_iter() {
        for var_id in perturbation.get_perturbed_vars().keys() {
            let variable = bn
                .as_graph()
                .find_variable(var_id.as_str())
                .ok_or_else(|| {
                    // This is unreachable in inference, but kept for completeness.
                    format!("Perturbation references unknown network variable `{var_id}`.")
                })?;
            variable_perturbations
                .entry(variable)
                .or_default()
                .push(perturbation_id.clone());
        }
    }

    for perturbation_ids in variable_perturbations.values_mut() {
        perturbation_ids.sort_by(|left, right| {
            perturbation_codes[left]
                .cmp(&perturbation_codes[right])
                .then_with(|| left.as_str().cmp(right.as_str()))
        });
    }

    Ok(variable_perturbations)
}

/// Multiplex the update function for a variable to handle all perturbations that set its value.
///
/// The update function for variable `v` that is perturbed by 2 perturbations `p_1: v = true` and
/// `p_2: v = false` is along the lines of:
///
/// `(p_1 -> f_v = true) & (!p_1 -> ((p_2 -> f_v = false) & (!p_2 -> f_v = original_update(v))))`
///
/// Our encoding of the perturbation selectors does not allow for them to be true at the same time.
///
/// TODO: Explore flattened encoding.
fn multiplex_update_for_variable(
    bn: &BooleanNetwork,
    selector_params: &[ParameterId],
    perturbation_codes: &HashMap<PerturbationId, SelectorCode>,
    perturbations: &PerturbationManager,
    variable: VariableId,
    perturbation_ids: &[PerturbationId],
    original_update: &FnUpdate,
) -> Result<FnUpdate, String> {
    let var_name = bn.get_variable_name(variable);
    let sketch_var_id = VarId::new(var_name).map_err(|e| {
        format!("Failed converting BN variable `{var_name}` to sketch variable ID: {e}.")
    })?;

    let mut update = original_update.clone();

    for perturbation_id in perturbation_ids {
        let code = *perturbation_codes
            .get(perturbation_id)
            .ok_or_else(|| format!("Perturbation `{perturbation_id}` has no selector code."))?;
        let perturbation = perturbations.get_perturbation(perturbation_id)?;
        let forced_value = *perturbation
            .get_perturbed_vars()
            .get(&sketch_var_id)
            .ok_or_else(|| {
                format!("Perturbation `{perturbation_id}` does not list variable `{var_name}`.")
            })?;

        let selector_matches = selector_matches_update(selector_params, code);
        let forced_update = if forced_value {
            FnUpdate::mk_true()
        } else {
            FnUpdate::mk_false()
        };
        update = selector_matches
            .clone()
            .implies(forced_update)
            .and(selector_matches.negation().implies(update));
    }

    Ok(update)
}

/// Build `FnUpdate` expression that is true exactly when the valuation of the perturbation selector
/// parameters matches the given selector code.
fn selector_matches_update(selector_params: &[ParameterId], code: SelectorCode) -> FnUpdate {
    let mut matches = FnUpdate::mk_true();
    for (bit_index, param) in selector_params.iter().enumerate() {
        let bit_is_set = (code >> bit_index) & 1 != 0;
        let param_update = FnUpdate::mk_param(*param, &[]);
        let bit_matches = if bit_is_set {
            param_update
        } else {
            param_update.negation()
        };
        matches = matches.and(bit_matches);
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketchbook::perturbations::{Perturbation, PerturbationManager};
    use biodivine_lib_param_bn::biodivine_std::traits::Set;
    use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
    use std::collections::BTreeMap;

    fn simple_bn() -> BooleanNetwork {
        BooleanNetwork::try_from("A -> A\nA -> B\n$A: A\n$B: A").unwrap()
    }

    #[test]
    fn no_perturbations_returns_unchanged_bn() {
        let bn = simple_bn();
        let manager = PerturbationManager::new_empty();
        let encoding = BnWithPerturbations::new(&manager, &bn).unwrap();

        assert_eq!(encoding.bn, bn);
        assert_eq!(encoding.num_perturbations(), 0);
        assert_eq!(encoding.code_for_wild_type(), 0);
        assert!(encoding.perturbation_codes.is_empty());
    }

    #[test]
    fn deterministic_selector_codes_for_sorted_ids() {
        let mut perturbed_a = BTreeMap::new();
        perturbed_a.insert(VarId::new("A").unwrap(), true);
        let mut perturbed_b = BTreeMap::new();
        perturbed_b.insert(VarId::new("B").unwrap(), false);

        let manager = PerturbationManager::new_from_perturbations(vec![
            ("pert_z", Perturbation::new("pert_z", perturbed_a)),
            ("pert_a", Perturbation::new("pert_a", perturbed_b)),
        ])
        .unwrap();

        let encoding = BnWithPerturbations::new(&manager, &simple_bn()).unwrap();
        assert_eq!(encoding.code_for_wild_type(), 0);
        assert_eq!(encoding.perturbation_codes.len(), 2);
        assert_eq!(
            encoding.perturbation_codes[&PerturbationId::new("pert_a").unwrap()],
            1
        );
        assert_eq!(
            encoding.perturbation_codes[&PerturbationId::new("pert_z").unwrap()],
            2
        );
        assert_eq!(encoding.selector_params.len(), 2);
    }

    #[test]
    fn multiplexes_true_and_false_constant_branches() {
        let mut perturbed_a = BTreeMap::new();
        perturbed_a.insert(VarId::new("A").unwrap(), true);
        let mut perturbed_b = BTreeMap::new();
        perturbed_b.insert(VarId::new("B").unwrap(), false);

        let manager = PerturbationManager::new_from_perturbations(vec![
            ("pert_a", Perturbation::new("pert_a", perturbed_a)),
            ("pert_b", Perturbation::new("pert_b", perturbed_b)),
        ])
        .unwrap();

        let encoding = BnWithPerturbations::new(&manager, &simple_bn()).unwrap();
        let a = encoding.bn.as_graph().find_variable("A").unwrap();
        let b = encoding.bn.as_graph().find_variable("B").unwrap();
        let a_update = encoding
            .bn
            .get_update_function(a)
            .as_ref()
            .unwrap()
            .to_string(&encoding.bn);
        let b_update = encoding
            .bn
            .get_update_function(b)
            .as_ref()
            .unwrap()
            .to_string(&encoding.bn);

        assert!(a_update.contains("true"));
        assert!(b_update.contains("false"));
        assert!(a_update.contains("pert_sel_"));
        assert!(b_update.contains("pert_sel_"));
    }

    #[test]
    fn rejects_unknown_network_variable() {
        let mut perturbed = BTreeMap::new();
        perturbed.insert(VarId::new("missing").unwrap(), true);
        let manager = PerturbationManager::new_from_perturbations(vec![(
            "pert_1",
            Perturbation::new("pert_1", perturbed),
        )])
        .unwrap();

        let err = BnWithPerturbations::new(&manager, &simple_bn()).unwrap_err();
        assert!(err.contains("unknown network variable"));
    }

    #[test]
    fn restrict_and_project_selector_colors() {
        let mut perturbed_a = BTreeMap::new();
        perturbed_a.insert(VarId::new("A").unwrap(), true);
        let mut perturbed_b = BTreeMap::new();
        perturbed_b.insert(VarId::new("B").unwrap(), false);
        let manager = PerturbationManager::new_from_perturbations(vec![
            ("pert_1", Perturbation::new("pert_1", perturbed_a)),
            ("pert_2", Perturbation::new("pert_2", perturbed_b)),
        ])
        .unwrap();

        let encoding = BnWithPerturbations::new(&manager, &simple_bn()).unwrap();
        let graph = SymbolicAsyncGraph::new(&encoding.bn).unwrap();
        let ctx = graph.symbolic_context();

        let pert_code = encoding.perturbation_codes[&PerturbationId::new("pert_1").unwrap()];
        let restricted = encoding.restrict_graph_to_perturbation(&graph, pert_code);
        assert!(!restricted.mk_unit_colors().is_empty());

        let projected =
            encoding.project_perturbation_selector_colors(ctx, &restricted.mk_unit_colors());
        let projected_again = encoding.project_perturbation_selector_colors(ctx, &projected);
        assert_eq!(projected.as_bdd(), projected_again.as_bdd());

        // Projecting the selector must leave other selector choices available for
        // subsequent dynamic-property evaluations.
        let candidate_vertices = GraphColoredVertices::new(projected.into_bdd(), ctx);
        let synchronized_graph = graph.restrict(&candidate_vertices);
        let second_code = encoding.perturbation_codes[&PerturbationId::new("pert_2").unwrap()];
        let second_restricted =
            encoding.restrict_graph_to_perturbation(&synchronized_graph, second_code);
        assert!(!second_restricted.mk_unit_colors().is_empty());
    }
}
