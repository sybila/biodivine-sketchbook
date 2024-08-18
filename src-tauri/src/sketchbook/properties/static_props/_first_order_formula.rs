use crate::algorithms::fo_logic::fol_tree::FolTreeNode;
use crate::algorithms::fo_logic::parser::parse_fol_formula;
use crate::sketchbook::model::ModelState;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A typesafe representation of a FOL formula used in static properties.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FirstOrderFormula {
    #[serde(
        serialize_with = "serialize_tree",
        deserialize_with = "deserialize_tree"
    )]
    tree: FolTreeNode,
}

/// *(internal)* Serialize field `tree` of `FirstOrderFormula` as a string.
fn serialize_tree<S>(tree: &FolTreeNode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&tree.to_string())
}

/// *(internal)* Deserialize field `tree` of `FirstOrderFormula` from the serialized string.
fn deserialize_tree<'de, D>(deserializer: D) -> Result<FolTreeNode, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_fol_formula(&s).map_err(de::Error::custom)
}

impl fmt::Display for FirstOrderFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tree)
    }
}

/// Creating first-order formulas.
impl FirstOrderFormula {
    /// Parse `FirstOrderFormula` instance directly from a string, which must be in a
    /// correct format.
    pub fn try_from_str(formula: &str) -> Result<FirstOrderFormula, String> {
        Ok(FirstOrderFormula {
            tree: parse_fol_formula(formula)?,
        })
    }
}

/// Editing first-order formulas.
impl FirstOrderFormula {
    /// Change the formula represented by this instance.
    pub fn change_formula(&mut self, new_formula: &str) -> Result<(), String> {
        self.tree = parse_fol_formula(new_formula)?;
        Ok(())
    }
}

/// Observing first-order formulas.
impl FirstOrderFormula {
    /// Str reference version of the first-order formula.
    pub fn as_str(&self) -> &str {
        &self.tree.formula_str
    }
}

/// Static methods (to check validity of formula strings).
impl FirstOrderFormula {
    /// Check if the formula is correctly formed based on predefined FOL syntactic rules.
    pub fn check_pure_syntax(formula: &str) -> Result<(), String> {
        let res = parse_fol_formula(formula);
        if res.is_ok() {
            Ok(())
        } else {
            Err(res.err().unwrap())
        }
    }

    /// Check if the formula is correctly formed based on predefined FO syntactic rules, and also
    /// whether formula respects the model (propositions must be valid entities in the model and
    /// so on).
    pub fn check_syntax_with_model(formula: String, model: &ModelState) -> Result<(), String> {
        println!("For now, {formula} cannot be checked with respect to the {model:?}.");
        todo!()
    }
}
