use crate::sketchbook::model::ModelState;
use crate::sketchbook::observations::{Dataset, Observation};
use crate::sketchbook::properties::dynamic_props::_mk_hctl_formulas::*;
use biodivine_hctl_model_checker::preprocessing::hctl_tree::HctlTreeNode;
use biodivine_hctl_model_checker::preprocessing::parser::{
    parse_and_minimize_hctl_formula, parse_hctl_formula,
};
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A typesafe representation of a HCTL formula used in dynamic properties.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct HctlFormula {
    #[serde(
        serialize_with = "serialize_tree",
        deserialize_with = "deserialize_tree"
    )]
    tree: HctlTreeNode,
}

/// *(internal)* Serialize field `tree` of `HctlFormula` as a string.
fn serialize_tree<S>(tree: &HctlTreeNode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&tree.to_string())
}

/// *(internal)* Deserialize field `tree` of `HctlFormula` from the serialized string.
fn deserialize_tree<'de, D>(deserializer: D) -> Result<HctlTreeNode, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_hctl_formula(&s).map_err(de::Error::custom)
}

impl fmt::Display for HctlFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tree)
    }
}

/// Creating hctl formulas.
impl HctlFormula {
    /// Parse `HctlFormula` object directly from a string, which must be in a correct format.
    /// We only check if the general HCTL syntax is correct, do not check proposition names
    /// validity.
    pub fn try_from_str(formula: &str) -> Result<HctlFormula, String> {
        Ok(HctlFormula {
            tree: parse_hctl_formula(formula)?,
        })
    }

    /// Encode an observation by a (propositional) formula depicting the corresponding state/sub-space.
    /// The observation's binary values are used to create a conjunction of literals.
    /// The `var_names` are used as propositions names in the formula.
    pub fn encode_observation(
        obs: &Observation,
        var_names: &[String],
    ) -> Result<HctlFormula, String> {
        let formula = encode_observation(obs, var_names)?;
        HctlFormula::try_from_str(&formula)
    }

    /// Encode each of the several observations, one by one.
    /// For details, see [Self::encode_observation].
    pub fn encode_multiple_observations(
        observations: &[Observation],
        var_names: &[String],
    ) -> Result<Vec<HctlFormula>, String> {
        let formulae = encode_multiple_observations(observations, var_names)?;
        formulae
            .iter()
            .map(|f| HctlFormula::try_from_str(f))
            .collect::<Result<Vec<HctlFormula>, String>>()
    }

    /// Encode a dataset of observations as a single HCTL formula. The particular formula
    /// template is chosen depending on the type of data (attractor data, time-series, ...).
    ///
    /// Only data with their type specified can be encoded.
    pub fn try_encode_dataset_hctl(dataset: &Dataset) -> Result<HctlFormula, String> {
        let formula = encode_dataset_hctl(dataset)?;
        HctlFormula::try_from_str(&formula)
    }
}

/// Editing HCTL formulas.
impl HctlFormula {
    /// Change the formula represented by this instance.
    pub fn change_formula(&mut self, new_formula: &str) -> Result<(), String> {
        self.tree = parse_hctl_formula(new_formula)?;
        Ok(())
    }
}

/// Observing HCTL formulas.
impl HctlFormula {
    /// Str reference version of the first-order formula.
    pub fn as_str(&self) -> &str {
        &self.tree.formula_str
    }
}

/// Static methods (to check validity of formula strings).
impl HctlFormula {
    /// Assert that formula is correctly formed based on HCTL syntactic rules.
    pub fn check_syntax(formula: &str) -> Result<(), String> {
        let res = parse_hctl_formula(formula);
        if res.is_ok() {
            Ok(())
        } else {
            Err(res.err().unwrap())
        }
    }

    /// Assert that formula is correctly formed based on HCTL syntactic rules, and also
    /// whether the propositions correspond to valid network variables used in the `model`.
    pub fn check_syntax_with_model(formula: &str, model: &ModelState) -> Result<(), String> {
        // create simplest bn possible, we just need to cover all the variables
        let bn = model.to_empty_bn();
        let ctx = SymbolicContext::new(&bn)?;

        let res: Result<HctlTreeNode, String> = parse_and_minimize_hctl_formula(&ctx, formula);
        if res.is_ok() {
            Ok(())
        } else {
            Err(res.err().unwrap())
        }
    }
}
