use crate::algorithms::eval_dynamic::processed_props::DataEncodingType;
use crate::sketchbook::ids::ObservationId;
use crate::sketchbook::observations::{Dataset, Observation, VarValue};
use crate::sketchbook::properties::HctlFormula;
use std::fmt::Write;

/// Encode a dataset of observations as a single HCTL formula. The particular formula
/// template is chosen depending on the type of data (attractor data, fixed-points, ...).
///
/// a) Fixed-point dataset is encoded as a conjunction of "steady-state formulas",
///    (see [mk_formula_fixed_point_list]) that ensures each observation correspond to a fixed point.
/// b) Attractor dataset is encoded as a conjunction of "attractor formulas",
///    (see [mk_formula_attractor_list]) that ensures each observation correspond to an attractor.
/// b) Trap-space dataset is encoded as a conjunction of "trap-space formulas",
///    (see [mk_formula_trap_space_list]) that ensures each observation correspond to a trap space.
pub fn encode_dataset_hctl_str(
    dataset: &Dataset,
    observation_id: Option<ObservationId>,
    category: DataEncodingType,
) -> Result<String, String> {
    let var_names = dataset
        .variables()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();

    let encoded_observations = if let Some(obs_id) = observation_id {
        let observation = dataset.get_observation(&obs_id)?;
        vec![encode_observation_str(observation, &var_names)?]
    } else {
        let observations = dataset.observations();
        encode_multiple_observations_str(observations, &var_names)?
    };

    match category {
        DataEncodingType::Attractor => Ok(mk_formula_attractor_list(&encoded_observations)),
        DataEncodingType::FixedPoint => Ok(mk_formula_fixed_point_list(&encoded_observations)),
        DataEncodingType::TrapSpace => Ok(mk_formula_trap_space_list(&encoded_observations)),
    }
}

/// Encode an observation by a (propositional) formula depicting the corresponding state/sub-space.
/// The observation's binary values are used to create a conjunction of literals.
/// The `var_names` are used as propositions names in the formula.
pub fn try_encode_observation(
    obs: &Observation,
    var_names: &[String],
) -> Result<HctlFormula, String> {
    let formula = encode_observation_str(obs, var_names)?;
    HctlFormula::try_from_str(&formula)
}

/// Encode each of the several observations, one by one.
/// For details, see [Self::encode_observation].
pub fn try_encode_multiple_observations(
    observations: &[Observation],
    var_names: &[String],
) -> Result<Vec<HctlFormula>, String> {
    let formulae = encode_multiple_observations_str(observations, var_names)?;
    formulae
        .iter()
        .map(|f| HctlFormula::try_from_str(f))
        .collect::<Result<Vec<HctlFormula>, String>>()
}

/// Encode binarized observation with a formula depicting the corresponding state/sub-space.
/// Using binarized values and proposition names, creates a conjunction of literals
/// describing that observation.
///
/// `00*1*1` would end up like `!v1 & !v2 & v4 & v6`
fn encode_observation_str(
    observation: &Observation,
    prop_names: &[String],
) -> Result<String, String> {
    if observation.num_values() != prop_names.len() {
        return Err("Numbers of observation's values and propositions differs.".to_string());
    }

    let formula: String = prop_names
        .iter()
        .enumerate()
        .filter_map(|(i, prop)| match observation.get_values()[i] {
            VarValue::True => Some(prop.to_string()),
            VarValue::False => Some(format!("~{prop}")),
            VarValue::Any => None,
        })
        .collect::<Vec<_>>()
        .join(" & ");

    // if observation corresponds to the whole space (all vars are '*'), we just use 'true'
    let final_formula = if formula.is_empty() {
        "(true)".to_string()
    } else {
        format!("({})", formula)
    };
    Ok(final_formula)
}

/// Encode several observation vectors with conjunction formulae, one by one.
/// Also see [encode_observation] for details.
fn encode_multiple_observations_str(
    observations: &[Observation],
    prop_names: &[String],
) -> Result<Vec<String>, String> {
    observations
        .iter()
        .map(|o| encode_observation_str(o, prop_names))
        .collect::<Result<Vec<String>, String>>()
}

/// Create HCTL formula describing that given specific state is part of an attractor.
///
/// > `EXISTS x. JUMP x. ({state} & AG EF {state})`
///
/// Arg `attractor_state` is a formula encoding the state of interest.
/// The state must be fully specified (conjunction of literals for EACH proposition).
pub fn mk_formula_attractor_specific(attractor_state: &str) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({attractor_state} & (AG EF ({attractor_state})))))")
}

/// Create HCTL formula describing that given sub-space (observation) must contain a state
/// that is part of an attractor.
///
/// This function works for sub-spaces in general, but if you have a singleton sub-space
/// (a state), we recommend using [mk_attractor_formula_specific] - is more optimized).
///
/// > `EXISTS x. JUMP x. ({state} & (AG EF ({state} & x)))`
///
/// Arg `attractor_state` is a formula encoding the sub-space with the state of interest.
pub fn mk_formula_attractor(attractor_state: &str) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({attractor_state} & (AG EF ({attractor_state} & {{x}})))))")
}

/// Create HCTL formula describing that each sub-space (observation) in a list must contain
/// a state that is part of an attractor.
/// It is essentially a conjunction of "attractor formulas" (see [mk_formula_attractor]).
///
/// > `ATTRACTOR({state1}) & ... & ATTRACTOR({stateN})`
///
/// Arg `attractor_state_list` is a vector of formulae, each encoding a sub-space
/// (conjunction of literals).
pub fn mk_formula_attractor_list(attractor_state_list: &[String]) -> String {
    assert!(!attractor_state_list.is_empty());

    let formula = attractor_state_list
        .iter()
        .map(|attractor_state| mk_formula_attractor(attractor_state))
        .collect::<Vec<_>>()
        .join(" & ");
    format!("({})", formula)
}

/// Create HCTL formula that prohibits existence of any attractor apart from the ones
/// that contain some states from some of the specified sub-spaces (observations).
///
/// > `! EXISTS x. JUMP x. !(AG EF ({state1} | ... | {stateN}))`
///
/// Arg `attractor_state_list` is a vector of formulae, each describing a state in particular
/// allowed attractor (conjunction of literals).
pub fn mk_formula_forbid_other_attractors(attractor_state_list: &[String]) -> String {
    assert!(!attractor_state_list.is_empty());

    let inner_disjunction = attractor_state_list
        .iter()
        .map(|attractor_state| format!("({attractor_state})"))
        .collect::<Vec<_>>()
        .join(" | ");

    format!("~(3{{x}}: (@{{x}}: ~(AG EF ({}))))", inner_disjunction)
}

/// Create HCTL formula describing that 1) each sub-space (observation) in a list must contain
/// a state that is part of an attractor and 2) prohibits existence of any additional attractor
/// that does not contain a state from some of these sub-spaces.
///
/// Basically a conjunction of two formulas, see [mk_formula_attractor_list] and
/// [mk_formula_forbid_other_attractors] for details.
///
/// > `ALL_ATTRACTORS({states}) & NO_OTHER_ATTRACTORS({states})`
pub fn mk_formula_exclusive_attractors(attractor_state_list: &[String]) -> String {
    assert!(!attractor_state_list.is_empty());
    let first_part = mk_formula_attractor_list(attractor_state_list);
    let second_part = mk_formula_forbid_other_attractors(attractor_state_list);
    format!("({first_part} & {second_part})")
}

/// Create a formula describing that a sub-space (observation) is a trap space.
///
/// > `FORALL x. JUMP x. ({sub_space} => ~(EX ~({sub_space})))`
///
/// Argument `sub_space` is a formula describing the sub-space of interest.
pub fn mk_formula_trap_space(sub_space: &str) -> String {
    assert!(!sub_space.is_empty());
    format!("(V{{x}}: (@{{x}}: ({sub_space} => ~(EX ~({sub_space})))))")
}

/// Create a formula describing that each sub-space (observation) in a given list is a trap space.
/// It essentially is a conjunction of "trap-space formulas" (see [mk_formula_trap_space]).
///
/// > `TRAP_SPACE({space1}) & ... & TRAP_SPACE({spaceN})`
pub fn mk_formula_trap_space_list(sub_spaces_list: &[String]) -> String {
    assert!(!sub_spaces_list.is_empty());

    let formula = sub_spaces_list
        .iter()
        .map(|sub_space| mk_formula_trap_space(sub_space))
        .collect::<Vec<_>>()
        .join(" & ");

    format!("({})", formula)
}

/// Create HCTL formula describing that given state is a steady state (fixed-point).
///
/// > `EXISTS x. JUMP x. ({state} & AX {state})`
///
/// Arg `steady_state` is a formula encoding the state of interest.
/// The state must be fully specified (conjunction of literals for EACH proposition).
pub fn mk_formula_fixed_point_specific(steady_state: &str) -> String {
    assert!(!steady_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({steady_state} & (AX ({steady_state})))))")
}

/// Create HCTL formula describing that given sub-space (observation) must contain a steady
/// state (fixed-point).
///
/// This function works for sub-spaces in general, but if you have a singleton sub-space
/// (a state), we recommend using [mk_formula_fixed_point_specific] - is more optimized).
///
/// > `EXISTS x. JUMP x. ({state} & (AX ({state} & x)))`
///
/// Arg `steady_state` is a formula encoding the sub-space with the state of interest.
pub fn mk_formula_fixed_point(steady_state: &str) -> String {
    assert!(!steady_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({steady_state} & (AX ({steady_state} & {{x}})))))")
}

/// Create HCTL formula describing that each sub-space (observation) in a list must contain
/// a steady state (fixed-point).
/// It is essentially a conjunction of "fixed-point formulas" (see [mk_formula_fixed_point]).
///
/// > `FIXED_POINT({state1}) & ... & FIXED_POINT({stateN})`
pub fn mk_formula_fixed_point_list(steady_state_list: &[String]) -> String {
    assert!(!steady_state_list.is_empty());

    let formula = steady_state_list
        .iter()
        .map(|steady_state| mk_formula_fixed_point(steady_state))
        .collect::<Vec<_>>()
        .join(" & ");

    format!("({})", formula)
}

/// Create HCTL formula that prohibits existence of any steady state apart from the ones
/// that contained in the specified sub-spaces (observations).
///
/// > `! EXISTS x. JUMP x. (!{state1} & ... & !{stateN} & (AX x)))`
///
/// Arg `steady_state_list` is a vector of formulae, each encoding particular allowed state.
pub fn mk_formula_forbid_other_fixed_points(steady_state_list: &[String]) -> String {
    assert!(!steady_state_list.is_empty());

    let inner_conjunction = steady_state_list
        .iter()
        .map(|steady_state| format!("~({})", steady_state))
        .collect::<Vec<_>>()
        .join(" & ");

    format!("~(3{{x}}: (@{{x}}: {} & (AX {{x}})))", inner_conjunction)
}

/// Create HCTL formula describing that 1) each sub-space (observation) in a list must contain
/// a steady state and 2) prohibits existence of any additional steady states outside of these
/// sub-spaces.
///
/// Basically a conjunction of two formulas, see [mk_formula_fixed_point_list] and
/// [mk_formula_forbid_other_attractors] for details.
///
/// > `FIXED_POINTS({states}) & NO_OTHER_FIXED_POINTS({states})`
///
/// Arg `steady_state_list` is a vector of formulae, each encoding one state of interested.
pub fn mk_formula_exclusive_fixed_points(steady_state_list: &[String]) -> String {
    assert!(!steady_state_list.is_empty());
    let first_part = mk_formula_fixed_point_list(steady_state_list);
    let second_part = mk_formula_forbid_other_fixed_points(steady_state_list);
    format!("({first_part} & {second_part})")
}

/// Create HCTL formula describing that there is (not) a path between (any) states of two
/// sub-spaces.
///
/// > positive: `EXISTS x. JUMP x. {from_state} & EF {to_state}`
/// > negative: `EXISTS x. JUMP x. {from_state} & !EF {to_state}`
///
/// `from_state` and `to_state` are both formulae encoding particular sub-spaces.
/// `is_negative` is true iff we want to encode non-existence of path.
pub fn mk_formula_reachability_pair(from_state: &str, to_state: &str, is_negative: bool) -> String {
    assert!(!to_state.is_empty() && !from_state.is_empty());
    if is_negative {
        return format!("(3{{x}}: (@{{x}}: {from_state} & (~EF ({to_state}))))");
    }
    format!("(3{{x}}: (@{{x}}: {from_state} & EF ({to_state})))")
}

/// Create a formula describing the existence of path between any states of every two consecutive
/// sub-spaces from the `states_sequence`, starting with the first one.
///
/// Basically, this can be used to describe a time series s0 -> s1 -> ... -> sN
///
/// > `EXISTS x. JUMP x. ({state1} & EF ({state2} & EF( ... )))`
pub fn mk_formula_reachability_chain(states_sequence: &[String]) -> String {
    let num_states = states_sequence.len();
    assert!(num_states > 0);

    let mut chain = String::new();
    for state in states_sequence.iter().take(num_states - 1) {
        write!(chain, "({}) & EF (", state).unwrap();
    }

    let final_state = &states_sequence[num_states - 1];
    let parentheses = ")".repeat(num_states - 1);
    write!(chain, "{}{}", final_state, parentheses).unwrap();

    format!("(3{{x}}: (@{{x}}: {}))", chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketchbook::observations::Observation;

    #[test]
    /// Test encoding of an observation.
    fn test_observation_encoding() {
        let prop_names = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];

        let obs1 = Observation::try_from_str("001*1", "o1").unwrap();
        let encoded1 = "(~a & ~b & c & e)";
        assert_eq!(
            encode_observation_str(&obs1, &prop_names).unwrap(),
            encoded1
        );

        let obs2 = Observation::try_from_str("001**", "o2").unwrap();
        let encoded2 = "(~a & ~b & c)";
        assert_eq!(
            encode_observation_str(&obs2, &prop_names).unwrap(),
            encoded2
        );

        let obs3 = Observation::try_from_str("*****", "o3").unwrap();
        let encoded3 = "(true)";
        assert_eq!(
            encode_observation_str(&obs3, &prop_names).unwrap(),
            encoded3
        );

        let multiple_encoded =
            encode_multiple_observations_str(&vec![obs1, obs2, obs3], &prop_names).unwrap();
        assert_eq!(multiple_encoded, vec![encoded1, encoded2, encoded3]);
    }

    #[test]
    /// Test generating different kinds of general attractor formulae.
    fn test_attractor_encodings() {
        let attr_states = vec!["a & b & ~c".to_string(), "a & b & c".to_string()];

        assert_eq!(
            &mk_formula_attractor_specific(&attr_states[0]),
            "(3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c)))))",
        );
        assert_eq!(
            &mk_formula_attractor(&attr_states[0]),
            "(3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c & {x})))))",
        );
        assert_eq!(
            &mk_formula_forbid_other_attractors(&attr_states),
            "~(3{x}: (@{x}: ~(AG EF ((a & b & ~c) | (a & b & c)))))",
        );
        assert_eq!(
            &mk_formula_attractor_list(&attr_states),
            "((3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AG EF (a & b & c & {x}))))))",
        );
        assert_eq!(
            &mk_formula_exclusive_attractors(&attr_states),
            "(((3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AG EF (a & b & c & {x})))))) & ~(3{x}: (@{x}: ~(AG EF ((a & b & ~c) | (a & b & c))))))",
        );
    }

    #[test]
    /// Test generating formulas for trap spaces.
    fn test_trap_space_encodings() {
        let sub_spaces = vec!["a & b & ~c".to_string(), "a & b & c".to_string()];

        let expected_formula = "(V{x}: (@{x}: (a & b & ~c => ~(EX ~(a & b & ~c)))))";
        assert_eq!(&mk_formula_trap_space(&sub_spaces[0]), expected_formula,);

        let expected_formula = "((V{x}: (@{x}: (a & b & ~c => ~(EX ~(a & b & ~c))))) & (V{x}: (@{x}: (a & b & c => ~(EX ~(a & b & c))))))";
        assert_eq!(mk_formula_trap_space_list(&sub_spaces), expected_formula);
    }

    #[test]
    /// Test generating of different kinds of steady-state formulae.
    fn test_fixed_point_encodings() {
        let attr_states = vec!["a & b & ~c".to_string(), "a & b & c".to_string()];

        assert_eq!(
            &mk_formula_fixed_point_specific(&attr_states[0]),
            "(3{x}: (@{x}: (a & b & ~c & (AX (a & b & ~c)))))",
        );
        assert_eq!(
            &mk_formula_fixed_point(&attr_states[0]),
            "(3{x}: (@{x}: (a & b & ~c & (AX (a & b & ~c & {x})))))",
        );
        assert_eq!(
            &mk_formula_forbid_other_fixed_points(&attr_states),
            "~(3{x}: (@{x}: ~(a & b & ~c) & ~(a & b & c) & (AX {x})))",
        );
        assert_eq!(
            &mk_formula_fixed_point_list(&attr_states),
            "((3{x}: (@{x}: (a & b & ~c & (AX (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AX (a & b & c & {x}))))))",
        );
        assert_eq!(
            &mk_formula_exclusive_fixed_points(&attr_states),
            "(((3{x}: (@{x}: (a & b & ~c & (AX (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AX (a & b & c & {x})))))) & ~(3{x}: (@{x}: ~(a & b & ~c) & ~(a & b & c) & (AX {x}))))",
        );
    }

    #[test]
    /// Test generating reachability formulae.
    fn test_reachability_encoding() {
        let states = vec![
            "a & b & ~c".to_string(),
            "a & b & c".to_string(),
            "~a & b & c".to_string(),
        ];

        assert_eq!(
            &mk_formula_reachability_pair(&states[0], &states[1], true),
            "(3{x}: (@{x}: a & b & ~c & (~EF (a & b & c))))",
        );
        assert_eq!(
            &mk_formula_reachability_pair(&states[0], &states[1], false),
            "(3{x}: (@{x}: a & b & ~c & EF (a & b & c)))",
        );
        assert_eq!(
            &mk_formula_reachability_chain(&states),
            "(3{x}: (@{x}: (a & b & ~c) & EF ((a & b & c) & EF (~a & b & c))))",
        );
    }
}
