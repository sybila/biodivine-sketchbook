use crate::algorithms::fo_logic::fol_tree::FolTreeNode;
use crate::algorithms::fo_logic::parser::{parse_and_minimize_fol_formula, parse_fol_formula};
use crate::algorithms::fo_logic::utils::get_var_from_implicit;
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
    /// Reference to a string form of the FOL formula.
    pub fn as_str(&self) -> &str {
        &self.tree.formula_str
    }

    /// Reference to a syntax tree of the first-order formula.
    pub fn tree(&self) -> &FolTreeNode {
        &self.tree
    }
}

/// Static methods (to check validity of formula strings).
impl FirstOrderFormula {
    /// Check if the formula is correctly formed based on predefined FOL syntactic rules.
    pub fn check_pure_syntax(formula: &str) -> Result<(), String> {
        // We use `parse_and_minimize_fol_formula` since it not only parses the formula, but
        // also validates the variable names (we dont care about the variable renaming step here).
        // We have to provide some placeholder name (for minimization), but it does not matter here
        parse_and_minimize_fol_formula(formula, "PLACEHOLDER").map(|_| ())
    }

    /// Check if the formula is correctly formed based on predefined FO syntactic rules, and
    /// also whether the atomic elements respect the model (e.g., function symbols must be valid
    /// given the `model` and so on).
    pub fn check_syntax_with_model(formula: &str, model: &ModelState) -> Result<(), String> {
        // We use `parse_and_minimize_fol_formula` since it not only parses the formula, but
        // also validates the variable names (we dont care about the variable renaming step here).
        // We have to provide some placeholder name (for minimization), but it does not matter here
        let tree = parse_and_minimize_fol_formula(formula, "PLACEHOLDER")?;

        // Check if all used functions symbols are valid for the model. A function symbol is valid
        // if it references an uninterpreted function or some update function.
        let function_symbols = tree.collect_unique_fn_symbols()?;
        for (fn_name, arity) in function_symbols.iter() {
            // Check if the name corresponds to an (implicit) update function symbol for some variable
            if let Ok(var) = get_var_from_implicit(fn_name) {
                // If this is an update fn symbol, the corresponding variable must exist,
                // and that the arity of the function is correct.
                if let Ok(valid_var_id) = model.get_var_id(&var) {
                    let update_fn_arity = model.regulators(&valid_var_id).unwrap().len(); // safe to unwrap
                    if update_fn_arity != *arity {
                        return Err(format!(
                            "Update function symbol `{fn_name}` is used with incorrect arity."
                        ));
                    }
                } else {
                    return Err(format!(
                        "There is no variable corresponding to update function `{fn_name}`."
                    ));
                }
            } else {
                // If this is not update fn symbol, it must be correspond to an uninterpreted function.
                if let Ok(valid_fn_id) = model.get_uninterpreted_fn_id(fn_name) {
                    let fn_arity = model.get_uninterpreted_fn_arity(&valid_fn_id).unwrap(); // safe to unwrap
                    if fn_arity != *arity {
                        return Err(format!(
                            "Function symbol `{fn_name}` is used with incorrect arity."
                        ));
                    }
                } else {
                    return Err(format!(
                        "Function `{fn_name}` with arity {arity} not found in the model."
                    ));
                }
            }
        }
        Ok(())
    }
}
