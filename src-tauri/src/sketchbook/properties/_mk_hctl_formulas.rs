use crate::sketchbook::observations::{DataCategory, Dataset, Observation, VarValue};

/// Encode binarized observation with a formula depicting the corresponding state/sub-space.
/// Using binarized values and proposition names, creates a conjunction of literals
/// describing that observation.
///
/// `00*1*1` would end up like `!v1 & !v2 & v4 & v6`
pub fn encode_observation(
    observation: &Observation,
    prop_names: &[String],
) -> Result<String, String> {
    if observation.num_values() != prop_names.len() {
        return Err("Numbers of observation's values and propositions differs.".to_string());
    }
    let mut formula = String::new();
    formula.push('(');

    for (i, prop) in prop_names.iter().enumerate() {
        match observation.get_values()[i] {
            VarValue::True => formula.push_str(format!("{prop} & ").as_str()),
            VarValue::False => formula.push_str(format!("~{prop} & ").as_str()),
            VarValue::Any => (),
        }
    }

    // formula might be 'empty' if all props can have arbitrary values - corresponding to 'true'
    if formula.len() == 1 {
        formula.push_str("true");
    } else {
        formula = formula.strip_suffix(" & ").unwrap().to_string();
    }
    formula.push(')');
    Ok(formula)
}

/// Encode several observation vectors with conjunction formulae, one by one.
/// Also see [encode_observation] for details.
pub fn encode_multiple_observations(
    observations: &[Observation],
    prop_names: &[String],
) -> Result<Vec<String>, String> {
    observations
        .iter()
        .map(|o| encode_observation(o, prop_names))
        .collect::<Result<Vec<String>, String>>()
}

/// Encode a dataset of observations as a single HCTL formula. The particular formula
/// template is chosen depending on the type of data (attractor data, time-series, ...).
///
/// Only data with their type specified can be encoded.
///
/// a) Time series are encoded as "reachability chain", see [mk_formula_reachability_chain].
/// b) Fixed-point dataset is encoded as a conjunction of "steady-state formulas",
///    (see [mk_formula_fixed_point_set]) that ensures each observation correspond to a fixed point.
/// c) Attractor dataset is encoded as a conjunction of "attractor formulas",
///    (see [mk_formula_attractor_set]) that ensures each observation correspond to an attractor.
pub fn encode_dataset_hctl(observation_list: &Dataset) -> Result<String, String> {
    let variables_strings = observation_list
        .variables()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    let encoded_observations =
        encode_multiple_observations(observation_list.observations(), &variables_strings)?;
    match observation_list.category() {
        DataCategory::Attractor => Ok(mk_formula_attractor_set(&encoded_observations)),
        DataCategory::FixedPoint => Ok(mk_formula_fixed_point_set(&encoded_observations)),
        DataCategory::TimeSeries => Ok(mk_formula_reachability_chain(&encoded_observations)),
        DataCategory::Unspecified => Err("Cannot encode data with unspecified type".to_string()),
    }
}

/// Create a formula describing the existence of a attractor containing specific state.
///
/// > `EXISTS x. JUMP x. ({state} & AG EF {state})`
///
/// Works only for FULLY described state (conjunction of literals for each proposition).
/// Param `attractor_state` is a formula describing a state in a desired attractor.
pub fn mk_formula_attractor_specific(attractor_state: &str) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({attractor_state} & (AG EF ({attractor_state})))))")
}

/// Create a formula describing the existence of a attractor containing partially specified state.
/// Works for both fully or partially described states (but for fully specified states, we
/// recommend using `mk_attractor_formula_specific`).
///
/// Formula is created in a way that the model-checker can detect the pattern and use AEON
/// algorithms to optimise its computation.
///
/// > `EXISTS x. JUMP x. ({state} & (BIND x. AG EF x))`
///
/// Param `attractor_state` is a formula describing a (partial) state in a desired attractor.
pub fn mk_formula_attractor_aeon(attractor_state: &str) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({attractor_state} & (!{{y}}: AG EF {{y}}))))")
}

/// Create a formula describing the existence of a attractor containing partially specified state.
///
/// Works correctly for both fully or partially described states (but for fully specified states,
/// we recommend using [mk_attractor_formula_specific] - can be more optimized).
///
/// > `EXISTS x. JUMP x. ({state} & (AG EF ({state} & x)))`
///
/// Param `attractor_state` is a formula describing a (partial) state in a desired attractor.
pub fn mk_formula_attractor(attractor_state: &str) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({attractor_state} & (AG EF ({attractor_state} & {{x}})))))")
}

/// Create a formula ensuring the existence of a set of attractor states. It is essentially
/// a conjunction of "attractor formulas" (see [mk_formula_attractor]).
///
/// > `ATTRACTOR({state1}) & ... & ATTRACTOR({stateN})`
pub fn mk_formula_attractor_set(attractor_state_set: &[String]) -> String {
    assert!(!attractor_state_set.is_empty());
    let mut formula = String::new();
    formula.push('(');
    for attractor_state in attractor_state_set {
        assert!(!attractor_state.is_empty());
        formula.push_str(mk_formula_attractor(attractor_state).as_str());
        formula.push_str(" & ");
    }
    formula = formula.strip_suffix(" & ").unwrap().to_string();
    formula.push(')');
    formula
}

/// Create a formula prohibiting existence of any attractor apart of the ones that
/// contain specified states.
///
/// > `! EXISTS x. JUMP x. !(AG EF ({state1} | ... | {stateN}))`
///
/// Param `attractor_state_set` is a vector of formulae, each describing a state in particular
/// allowed attractor (conjunction of literals).
pub fn mk_formula_forbid_other_attractors(attractor_state_set: &[String]) -> String {
    assert!(!attractor_state_set.is_empty());
    let mut formula = String::new();
    formula.push_str("~(3{x}: (@{x}: ~(AG EF (");
    for attractor_state in attractor_state_set {
        assert!(!attractor_state.is_empty());
        formula.push_str(format!("({attractor_state}) | ").as_str())
    }
    formula = formula.strip_suffix(" | ").unwrap().to_string();
    formula.push_str("))))");
    formula
}

/// Create a formula ensuring the existence of a set of attractor states and prohibiting any
/// other attractors not containing these states.
///
/// Basically a conjunction of two formulas, see [mk_formula_attractor_set] and
/// [mk_formula_forbid_other_attractors] for details.
///
/// > `ALL_ATTRACTORS({states}) & NO_OTHER_ATTRACTORS({states})`
pub fn mk_formula_exclusive_attractors(attractor_state_set: &[String]) -> String {
    assert!(!attractor_state_set.is_empty());
    let first_part = mk_formula_attractor_set(attractor_state_set);
    let second_part = mk_formula_forbid_other_attractors(attractor_state_set);
    format!("({first_part} & {second_part})")
}

/// Create a formula describing the existence of a specific steady-state.
/// Works only for FULLY described states (conjunction with a literal for each proposition).
///
/// > `EXISTS x. JUMP x. ({state} & AX {state})`
///
/// Param `steady_state` is a formula describing that particular state.
pub fn mk_formula_fixed_point_specific(steady_state: &str) -> String {
    assert!(!steady_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({steady_state} & (AX ({steady_state})))))")
}

/// Create a formula describing the existence of a (partially specified) steady-state.
///
/// Works correctly for both fully or partially described states (but for fully specified states,
/// we recommend using [mk_formula_fixed_point_specific] - can be more optimized).
///
/// > `EXISTS x. JUMP x. ({state} & (AX ({state} & x)))`
///
/// Param `steady_state` is a formula describing that particular state.
pub fn mk_formula_fixed_point(steady_state: &str) -> String {
    assert!(!steady_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({steady_state} & (AX ({steady_state} & {{x}})))))")
}

/// Create a formula ensuring the existence of a set of fixed points. It is essentially
/// a conjunction of "fixed-point formulas" (see [mk_formula_fixed_point]).
///
/// > `FIXED_POINT({state1}) & ... & FIXED_POINT({stateN})`
pub fn mk_formula_fixed_point_set(steady_state_set: &[String]) -> String {
    let mut formula = String::new();
    formula.push('(');
    for steady_state in steady_state_set {
        formula.push_str(mk_formula_fixed_point(steady_state).as_str());
        formula.push_str(" & ");
    }
    formula = formula.strip_suffix(" & ").unwrap().to_string();
    formula.push(')');
    formula
}

/// Create a formula prohibiting all but the given states to be fixed-points.
///
/// Param `steady_state_set` is a vector of formulae, each describing particular allowed state.
///
/// > `! EXISTS x. JUMP x. (!{state1} & ... & !{stateN} & (AX x)))`
pub fn mk_formula_forbid_other_fixed_points(steady_state_set: &[String]) -> String {
    let mut formula = String::new();
    formula.push_str("~(3{x}: (@{x}: ");
    for steady_state in steady_state_set {
        assert!(!steady_state.is_empty());
        formula.push_str(format!("~({steady_state}) & ").as_str())
    }
    formula.push_str("(AX {x})))");
    formula
}

/// Create a formula ensuring the existence of a set of fixed points and prohibiting all other
/// states to be fixed-points.
///
/// Basically a conjunction of two formulas, see [mk_formula_fixed_point_set] and
/// [mk_formula_forbid_other_attractors] for details.
///
/// > `FIXED_POINTS({states}) & NO_OTHER_FIXED_POINTS({states})`
///
/// Param `steady_state_set` is a vector of formulae, each describing one state.
pub fn mk_formula_exclusive_fixed_points(steady_state_set: &[String]) -> String {
    assert!(!steady_state_set.is_empty());
    let first_part = mk_formula_fixed_point_set(steady_state_set);
    let second_part = mk_formula_forbid_other_fixed_points(steady_state_set);
    format!("({first_part} & {second_part})")
}

/// Create a formula describing the (non)existence of reachability between two (partial) states.
///
/// > positive: `EXISTS x. JUMP x. {from_state} & EF {to_state}`
/// > negative: `EXISTS x. JUMP x. {from_state} & !EF {to_state}`
///
/// `from_state` and `to_state` are both formulae describing particular states.
/// `is_negative` is true iff we want to non-existence of path from `from_state` to `to_state`
pub fn mk_formula_reachability_pair(from_state: &str, to_state: &str, is_negative: bool) -> String {
    assert!(!to_state.is_empty() && !from_state.is_empty());
    if is_negative {
        return format!("(3{{x}}: (@{{x}}: {from_state} & (~EF ({to_state}))))");
    }
    format!("(3{{x}}: (@{{x}}: {from_state} & EF ({to_state})))")
}

/// Create a formula describing the existence of reachability between every two consecutive states
/// from the `states_sequence`, starting with the first one.
///
/// Basically can be used to describe a time series s0 -> s1 -> ... -> sN
///
/// > `EXISTS x. JUMP x. ({state1} & EF ({state2} & EF( ... )))`
pub fn mk_formula_reachability_chain(states_sequence: &[String]) -> String {
    let mut formula = String::new();
    formula.push_str("(3{x}: (@{x}: ");
    let num_states = states_sequence.len();
    for (n, state) in states_sequence.iter().enumerate() {
        assert!(!state.is_empty());
        if n == num_states - 1 {
            break;
        }
        formula.push_str(format!("({state}) & EF (").as_str())
    }

    // add the last state and all the closing parentheses
    formula.push_str(states_sequence[num_states - 1].to_string().as_str());
    let parentheses = (0..num_states + 1).map(|_| ")").collect::<String>();
    formula.push_str(parentheses.as_str());
    formula
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::observations::{DataCategory, Dataset, Observation};
    use crate::sketchbook::properties::_mk_hctl_formulas::*;

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
        assert_eq!(encode_observation(&obs1, &prop_names).unwrap(), encoded1);

        let obs2 = Observation::try_from_str("001**", "o2").unwrap();
        let encoded2 = "(~a & ~b & c)";
        assert_eq!(encode_observation(&obs2, &prop_names).unwrap(), encoded2);

        let obs3 = Observation::try_from_str("*****", "o3").unwrap();
        let encoded3 = "(true)";
        assert_eq!(encode_observation(&obs3, &prop_names).unwrap(), encoded3);

        let multiple_encoded =
            encode_multiple_observations(&vec![obs1, obs2, obs3], &prop_names).unwrap();
        assert_eq!(multiple_encoded, vec![encoded1, encoded2, encoded3]);
    }

    #[test]
    /// Test encodings various kinds of datasets.
    fn test_dataset_encoding() {
        let observation1 = Observation::try_from_str("110", "obs1").unwrap();
        let observation2 = Observation::try_from_str("1*1", "obs2").unwrap();
        let raw_observations = vec![observation1, observation2];
        let var_names = vec!["a", "b", "c"];

        let attr_observations = Dataset::new(
            raw_observations.clone(),
            var_names.clone(),
            DataCategory::Attractor,
        );
        assert_eq!(
            &encode_dataset_hctl(&attr_observations.unwrap()).unwrap(),
            "((3{x}: (@{x}: ((a & b & ~c) & (AG EF ((a & b & ~c) & {x}))))) & (3{x}: (@{x}: ((a & c) & (AG EF ((a & c) & {x}))))))",
        );

        let fixed_point_observations = Dataset::new(
            raw_observations.clone(),
            var_names.clone(),
            DataCategory::FixedPoint,
        );
        assert_eq!(
            &encode_dataset_hctl(&fixed_point_observations.unwrap()).unwrap(),
            "((3{x}: (@{x}: ((a & b & ~c) & (AX ((a & b & ~c) & {x}))))) & (3{x}: (@{x}: ((a & c) & (AX ((a & c) & {x}))))))",
        );

        let time_series_observations = Dataset::new(
            raw_observations.clone(),
            var_names.clone(),
            DataCategory::TimeSeries,
        );
        assert_eq!(
            &encode_dataset_hctl(&time_series_observations.unwrap()).unwrap(),
            "(3{x}: (@{x}: ((a & b & ~c)) & EF ((a & c))))",
        );

        let unspecified_observations = Dataset::new(
            raw_observations.clone(),
            var_names.clone(),
            DataCategory::Unspecified,
        );
        assert!(encode_dataset_hctl(&unspecified_observations.unwrap()).is_err());
    }

    #[test]
    /// Test generating of different kinds of general attractor formulae.
    fn test_attractor_encodings() {
        let attr_states = vec!["a & b & ~c".to_string(), "a & b & c".to_string()];

        assert_eq!(
            &mk_formula_attractor_specific(&attr_states[0]),
            "(3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c)))))",
        );
        assert_eq!(
            &mk_formula_attractor_aeon(&attr_states[0]),
            "(3{x}: (@{x}: (a & b & ~c & (!{y}: AG EF {y}))))",
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
            &mk_formula_attractor_set(&attr_states),
            "((3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AG EF (a & b & c & {x}))))))",
        );
        assert_eq!(
            &mk_formula_exclusive_attractors(&attr_states),
            "(((3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AG EF (a & b & c & {x})))))) & ~(3{x}: (@{x}: ~(AG EF ((a & b & ~c) | (a & b & c))))))",
        );
    }

    #[test]
    /// Test generating of different kinds of steady state formulae.
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
            &mk_formula_fixed_point_set(&attr_states),
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
