use crate::sketchbook::observations::{DataCategory, Dataset, Observation, VarValue};

/// Encode binarized observation with a formula depicting the corresponding state/sub-space.
/// Using binarized values and proposition names, creates a conjunction of literals
/// describing that observation.
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
pub fn encode_observation_list_hctl(observation_list: &Dataset) -> Result<String, String> {
    let variables_strings = observation_list
        .variables()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    let encoded_observations =
        encode_multiple_observations(observation_list.observations(), &variables_strings)?;
    match observation_list.category() {
        DataCategory::Attractor => Ok(mk_formula_attractor_set(encoded_observations)),
        DataCategory::FixedPoint => Ok(mk_formula_fixed_point_set(encoded_observations)),
        DataCategory::TimeSeries => Ok(mk_formula_reachability_chain(encoded_observations)),
        DataCategory::Unspecified => Err("Cannot encode data with unspecified type".to_string()),
    }
}

/// Create a formula describing the existence of a attractor containing specific state.
///
/// Works only for FULLY described state (conjunction of literals for each proposition).
/// Param `attractor_state` is a formula describing a state in a desired attractor.
pub fn mk_formula_attractor_specific(attractor_state: String) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({attractor_state} & (AG EF ({attractor_state})))))")
}

/// Create a formula describing the existence of a attractor containing partially specified state.
/// Works for both fully or partially described states (but for fully specified states, we
/// recommend using `mk_attractor_formula_specific`).
///
/// Formula is created in a way that the model-checker can use AEON algorithms to optimise its
/// computation.
///
/// Param `attractor_state` is a formula describing a (partial) state in a desired attractor.
pub fn mk_formula_attractor_aeon(attractor_state: String) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({attractor_state} & (!{{y}}: AG EF {{y}}))))")
}

/// Create a formula describing the existence of a attractor containing partially specified state.
///
/// Works for both fully or partially described states (but for fully specified states, we
/// recommend using `mk_attractor_formula_specific`).
///
/// Param `attractor_state` is a formula describing a (partial) state in a desired attractor.
pub fn mk_formula_attractor(attractor_state: String) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({attractor_state} & (AG EF ({attractor_state} & {{x}})))))")
}

/// Create a formula ensuring the existence of a set of attractor states.
pub fn mk_formula_attractor_set(attractor_state_set: Vec<String>) -> String {
    assert!(!attractor_state_set.is_empty());
    let mut formula = String::new();
    formula.push('(');
    for attractor_state in attractor_state_set {
        formula.push_str(mk_formula_attractor(attractor_state).as_str());
        formula.push_str(" & ");
    }
    formula = formula.strip_suffix(" & ").unwrap().to_string();
    formula.push(')');
    formula
}

/// Create a formula prohibiting all attractors that do not contain specified states.
///
/// Param `attractor_state_set` is a vector of formulae, each describing a state in particular
/// allowed attractor (conjunction of literals).
pub fn mk_formula_forbid_other_attractors(attractor_state_set: Vec<String>) -> String {
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
pub fn mk_formula_exclusive_attractors(attractor_state_set: Vec<String>) -> String {
    // part which ensures attractor states
    let mut formula = String::new();
    for attractor_state in attractor_state_set.clone() {
        formula.push_str(mk_formula_attractor(attractor_state).as_str());
        formula.push_str(" & ");
    }

    // append the sub-formula which forbids additional attractor states
    formula.push_str(mk_formula_forbid_other_attractors(attractor_state_set).as_str());
    formula
}

/// Create a formula describing the existence of a specific steady-state.
///
/// Works only for FULLY described states (conjunction with a literal for each proposition).
/// Param `steady_state` is a formula describing that particular state.
pub fn mk_formula_fixed_point_specific(steady_state: String) -> String {
    assert!(!steady_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({steady_state} & (AX ({steady_state})))))")
}

/// Create a formula describing the existence of a (partially specified) steady-state.
///
/// Works for both fully or partially specified described states.
/// Param `steady_state` is a formula describing that particular state.
pub fn mk_formula_fixed_point(steady_state: String) -> String {
    assert!(!steady_state.is_empty());
    format!("(3{{x}}: (@{{x}}: ({steady_state} & (AX ({steady_state} & {{x}})))))")
}

/// Create a formula ensuring the existence of a set of fixed points.
pub fn mk_formula_fixed_point_set(steady_state_set: Vec<String>) -> String {
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
pub fn mk_formula_forbid_other_fixed_points(steady_state_set: Vec<String>) -> String {
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
/// This formula is build in a way that uses advantage of model-checkers cashing (for "AX x").
/// Param `steady_state_set` is a vector of formulae, each describing one state.
pub fn mk_formula_exclusive_fixed_points(steady_state_set: Vec<String>) -> String {
    // part which ensures steady states
    let mut formula = String::new();
    for steady_state in steady_state_set.clone() {
        formula.push_str(mk_formula_fixed_point(steady_state).as_str());
        formula.push_str(" & ");
    }

    // append the sub-formula which forbids additional steady states
    formula.push_str(mk_formula_forbid_other_fixed_points(steady_state_set).as_str());
    formula
}

/// Create a formula describing the (non)existence of reachability between two (partial) states.
///
/// `from_state` and `to_state` are both formulae describing particular states.
/// `is_negative` is true iff we want to non-existence of path from `from_state` to `to_state`
pub fn mk_formula_reachability_pair(
    from_state: String,
    to_state: String,
    is_negative: bool,
) -> String {
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
pub fn mk_formula_reachability_chain(states_sequence: Vec<String>) -> String {
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
    formula.push_str(
        (0..num_states + 1)
            .map(|_| ")")
            .collect::<String>()
            .as_str(),
    );
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

        let observation1 = Observation::try_from_str("001*1", "obs1").unwrap();
        let encoded1 = "(~a & ~b & c & e)";
        assert_eq!(
            encode_observation(&observation1, &prop_names).unwrap(),
            encoded1
        );

        let observation2 = Observation::try_from_str("001**", "obs2").unwrap();
        let encoded2 = "(~a & ~b & c)";
        assert_eq!(
            encode_observation(&observation2, &prop_names).unwrap(),
            encoded2
        );

        let observation3 = Observation::try_from_str("*****", "obs3").unwrap();
        let encoded3 = "(true)";
        assert_eq!(
            encode_observation(&observation3, &prop_names).unwrap(),
            encoded3
        );

        assert_eq!(
            encode_multiple_observations(
                &vec![observation1, observation2, observation3],
                &prop_names
            )
            .unwrap(),
            vec![encoded1, encoded2, encoded3]
        );
    }

    #[test]
    /// Test encoding of a list of observations of various kinds.
    fn test_attractor_observations_encoding() {
        let observation1 = Observation::try_from_str("110", "obs1").unwrap();
        let observation2 = Observation::try_from_str("1*1", "obs2").unwrap();
        let raw_observations = vec![observation1, observation2];
        let var_names = vec!["a", "b", "c"];

        let attr_observations = Dataset::new(
            raw_observations.clone(),
            var_names.clone(),
            DataCategory::Attractor,
        )
        .unwrap();
        assert_eq!(
            encode_observation_list_hctl(&attr_observations).unwrap(),
            "((3{x}: (@{x}: ((a & b & ~c) & (AG EF ((a & b & ~c) & {x}))))) & (3{x}: (@{x}: ((a & c) & (AG EF ((a & c) & {x}))))))".to_string(),
        );

        let fixed_point_observations = Dataset::new(
            raw_observations.clone(),
            var_names.clone(),
            DataCategory::FixedPoint,
        )
        .unwrap();
        assert_eq!(
            encode_observation_list_hctl(&fixed_point_observations).unwrap(),
            "((3{x}: (@{x}: ((a & b & ~c) & (AX ((a & b & ~c) & {x}))))) & (3{x}: (@{x}: ((a & c) & (AX ((a & c) & {x}))))))".to_string(),
        );

        let time_series_observations = Dataset::new(
            raw_observations.clone(),
            var_names.clone(),
            DataCategory::TimeSeries,
        )
        .unwrap();
        assert_eq!(
            encode_observation_list_hctl(&time_series_observations).unwrap(),
            "(3{x}: (@{x}: ((a & b & ~c)) & EF ((a & c))))".to_string(),
        );

        let unspecified_observations = Dataset::new(
            raw_observations.clone(),
            var_names.clone(),
            DataCategory::Unspecified,
        )
        .unwrap();
        assert!(encode_observation_list_hctl(&unspecified_observations).is_err());
    }

    #[test]
    /// Test generating of different kinds of general attractor formulae.
    fn test_attractor_encodings() {
        let attr_states = vec!["a & b & ~c".to_string(), "a & b & c".to_string()];

        assert_eq!(
            mk_formula_attractor_specific(attr_states[0].clone()),
            "(3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c)))))".to_string(),
        );
        assert_eq!(
            mk_formula_attractor_aeon(attr_states[0].clone()),
            "(3{x}: (@{x}: (a & b & ~c & (!{y}: AG EF {y}))))".to_string(),
        );
        assert_eq!(
            mk_formula_attractor(attr_states[0].clone()),
            "(3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c & {x})))))".to_string(),
        );
        assert_eq!(
            mk_formula_forbid_other_attractors(attr_states.clone()),
            "~(3{x}: (@{x}: ~(AG EF ((a & b & ~c) | (a & b & c)))))".to_string(),
        );
        assert_eq!(
            mk_formula_attractor_set(attr_states.clone()),
            "((3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AG EF (a & b & c & {x}))))))".to_string(),
        );
        assert_eq!(
            mk_formula_exclusive_attractors(attr_states.clone()),
            "(3{x}: (@{x}: (a & b & ~c & (AG EF (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AG EF (a & b & c & {x}))))) & ~(3{x}: (@{x}: ~(AG EF ((a & b & ~c) | (a & b & c)))))".to_string(),
        );
    }

    #[test]
    /// Test generating of different kinds of steady state formulae.
    fn test_fixed_point_encodings() {
        let attr_states = vec!["a & b & ~c".to_string(), "a & b & c".to_string()];

        assert_eq!(
            mk_formula_fixed_point_specific(attr_states[0].clone()),
            "(3{x}: (@{x}: (a & b & ~c & (AX (a & b & ~c)))))".to_string(),
        );
        assert_eq!(
            mk_formula_fixed_point(attr_states[0].clone()),
            "(3{x}: (@{x}: (a & b & ~c & (AX (a & b & ~c & {x})))))".to_string(),
        );
        assert_eq!(
            mk_formula_forbid_other_fixed_points(attr_states.clone()),
            "~(3{x}: (@{x}: ~(a & b & ~c) & ~(a & b & c) & (AX {x})))".to_string(),
        );
        assert_eq!(
            mk_formula_fixed_point_set(attr_states.clone()),
            "((3{x}: (@{x}: (a & b & ~c & (AX (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AX (a & b & c & {x}))))))".to_string(),
        );
        assert_eq!(
            mk_formula_exclusive_fixed_points(attr_states.clone()),
            "(3{x}: (@{x}: (a & b & ~c & (AX (a & b & ~c & {x}))))) & (3{x}: (@{x}: (a & b & c & (AX (a & b & c & {x}))))) & ~(3{x}: (@{x}: ~(a & b & ~c) & ~(a & b & c) & (AX {x})))".to_string(),
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
            mk_formula_reachability_pair(states[0].clone(), states[1].clone(), true),
            "(3{x}: (@{x}: a & b & ~c & (~EF (a & b & c))))".to_string(),
        );
        assert_eq!(
            mk_formula_reachability_pair(states[0].clone(), states[1].clone(), false),
            "(3{x}: (@{x}: a & b & ~c & EF (a & b & c)))".to_string(),
        );
        assert_eq!(
            mk_formula_reachability_chain(states),
            "(3{x}: (@{x}: (a & b & ~c) & EF ((a & b & c) & EF (~a & b & c))))".to_string(),
        );
    }
}
