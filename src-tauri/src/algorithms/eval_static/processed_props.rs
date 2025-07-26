use crate::algorithms::eval_static::encode::*;
use crate::algorithms::fo_logic::fol_tree::FolTreeNode;
use crate::algorithms::fo_logic::parser::parse_and_minimize_fol_formula;
use crate::sketchbook::model::FnTree;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::Sketch;

use biodivine_lib_param_bn::BooleanNetwork;
use std::collections::HashMap;

/// Processed static property encoded in FOL.
///
/// For now, we use FOL checking as the only evaluation method, so we only need a single
/// struct for processed static properties. If we ever add more evaluation methods, we
/// can make this struct into an enum (like the one we use for dynamic props).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedStatProp {
    pub id: String,
    pub formula: FolTreeNode,
}

/// Simplified constructors for processed dynamic properties.
impl ProcessedStatProp {
    /// Create FOL `ProcessedStatProp` instance.
    pub fn mk_fol(id: &str, formula: FolTreeNode) -> ProcessedStatProp {
        ProcessedStatProp {
            id: id.to_string(),
            formula,
        }
    }

    /// Get ID of the processed property.
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Process static properties from a sketch, converting them into a more suitable
/// version that will be used for the evaluation. Currently, all properties are encoded
/// into FOL formulas, but we might support some other kind of preprocessing in future.
///
/// Note that since some uninterpreted functions have specified logical expressions, they
/// are pruned out from the BN, and we need to substitute them with their expression. The
/// expressions for substitution are provided in the `fn_expressions` mapping. All functions
/// must be present in the map - if there is no expression, None has to be mapped.
///
/// Properties must not reference "redundant" (completely unused in update functions) fn symbols
/// that were filtered out from the BN. This should be enforced by the consistency check before.
///
/// Arg `base_var_name` is used to rename extra FOL variable to a canonical form.
pub fn process_static_props(
    sketch: &Sketch,
    bn: &BooleanNetwork,
    fn_expressions: HashMap<String, Option<FnTree>>,
    base_var_name: &str,
) -> Result<Vec<ProcessedStatProp>, String> {
    // Convert the fn expressions between different structs. The expression is originally
    // Boolean, but we need to treat it as first-order for the substitution into FOL properties.
    let fn_expressions = fn_expressions
        .into_iter()
        .map(|(fn_id, fn_expression)| (fn_id, fn_expression.map(FolTreeNode::from_fn_tree)))
        .collect::<HashMap<String, Option<FolTreeNode>>>();

    let mut static_props = sketch.properties.stat_props().collect::<Vec<_>>();
    // Sort properties by IDs for deterministic computation order (and get rid of the IDs)
    static_props.sort_by(|(a_id, _), (b_id, _)| a_id.cmp(b_id));

    let mut processed_props = Vec::new();
    for (id, stat_prop) in static_props {
        // Everything is currently encoded into first-order logic formula
        let parsed_fol_expression = match stat_prop.get_prop_data() {
            StatPropertyType::GenericStatProp(prop) => {
                // Take the formula and fully process it (may not have been fully processed before)
                let formula = prop.processed_formula.as_str();
                let parsed_tree = parse_and_minimize_fol_formula(formula, base_var_name)?;
                // Replace pruned fn symbols in the formula with their expressions
                parsed_tree.substitute_fns_with_expressions(&fn_expressions)
            }
            StatPropertyType::RegulationEssential(prop)
            | StatPropertyType::RegulationEssentialContext(prop) => {
                let input_name = prop.input.clone().unwrap();
                let target_name = prop.target.clone().unwrap();
                // Encode the regulation property into FOL formula
                let mut formula = encode_regulation_essentiality(
                    input_name.as_str(),
                    target_name.as_str(),
                    prop.clone().value,
                    bn,
                );
                // Add context formula if provided
                if let Some(context_formula) = &prop.context {
                    formula = encode_property_in_context(context_formula, &formula);
                }
                parse_and_minimize_fol_formula(&formula, base_var_name)?
            }
            StatPropertyType::RegulationMonotonic(prop)
            | StatPropertyType::RegulationMonotonicContext(prop) => {
                let input_name = prop.input.clone().unwrap();
                let target_name = prop.target.clone().unwrap();
                // Encode the regulation property into FOL formula
                let mut formula = encode_regulation_monotonicity(
                    input_name.as_str(),
                    target_name.as_str(),
                    prop.clone().value,
                    bn,
                );
                // Add context formula if provided
                if let Some(context_formula) = &prop.context {
                    formula = encode_property_in_context(context_formula, &formula);
                }
                parse_and_minimize_fol_formula(&formula, base_var_name)?
            }
            StatPropertyType::FnInputEssential(prop)
            | StatPropertyType::FnInputEssentialContext(prop) => {
                let fn_id = prop.target.clone().unwrap();
                // Encode the function property into FOL formula. If the function symbol
                // has specified expression for substitution, replace it after.
                let input_idx = prop.input_index.unwrap();
                let number_inputs = sketch.model.get_uninterpreted_fn_arity(&fn_id)?;
                let mut formula = encode_essentiality(
                    number_inputs,
                    input_idx,
                    fn_id.as_str(),
                    prop.clone().value,
                );
                // Add context formula if provided
                if let Some(context_formula) = &prop.context {
                    formula = encode_property_in_context(context_formula, &formula);
                }
                let parsed_tree = parse_and_minimize_fol_formula(&formula, base_var_name)?;
                // Replace pruned fn symbols in the formula with their expressions
                parsed_tree.substitute_fns_with_expressions(&fn_expressions)
            }
            StatPropertyType::FnInputMonotonic(prop)
            | StatPropertyType::FnInputMonotonicContext(prop) => {
                let fn_id = prop.target.clone().unwrap();
                // Encode the function property into FOL formula. If the function symbol
                // has specified expression for substitution, replace it after.
                let input_idx = prop.input_index.unwrap();
                let number_inputs = sketch.model.get_uninterpreted_fn_arity(&fn_id)?;
                let mut formula = encode_monotonicity(
                    number_inputs,
                    input_idx,
                    fn_id.as_str(),
                    prop.clone().value,
                );
                // Add context formula if provided
                if let Some(context_formula) = &prop.context {
                    formula = encode_property_in_context(context_formula, &formula);
                }
                let parsed_tree = parse_and_minimize_fol_formula(&formula, base_var_name)?;
                // Replace pruned fn symbols in the formula with their expressions
                parsed_tree.substitute_fns_with_expressions(&fn_expressions)
            }
        };
        let new_prop = ProcessedStatProp::mk_fol(id.as_str(), parsed_fol_expression);
        processed_props.push(new_prop);
    }

    Ok(processed_props)
}

#[cfg(test)]
mod tests {
    use crate::algorithms::eval_static::processed_props::{
        process_static_props, ProcessedStatProp,
    };
    use crate::algorithms::fo_logic::fol_tree::FolTreeNode;
    use crate::algorithms::fo_logic::operator_enums::{Quantifier, UnaryOp};
    use crate::inference::inference_solver::InferenceSolver;
    use crate::sketchbook::model::{BinaryOp, Essentiality};
    use crate::sketchbook::{properties::StatProperty, Sketch};

    #[test]
    /// Test processing of a simple FOL property that does not require function
    /// expression substitutions.
    fn test_process_fol_prop_simple() {
        // Create basic sketch with no regulation props, single variable `A`
        // and fn symbol `f` (with no expression)
        let aeon_str = "A-??A\n$A:f(A)";
        let mut sketch = Sketch::from_aeon(aeon_str).unwrap();

        // Add simple FOL property involving function f
        let prop = StatProperty::try_mk_generic("p1", "\\forall x: f(x)").unwrap();
        sketch.properties.add_static_by_str("p1", prop).unwrap();

        // Process the static properties and check results
        // The formula stays the same, only the variable's name is standardized
        let (bn, expressions_map) = InferenceSolver::extract_bn(&sketch).unwrap();
        let processed_props = process_static_props(&sketch, &bn, expressions_map, "A").unwrap();
        let formula_expected = FolTreeNode::mk_quantifier(
            FolTreeNode::mk_function("f", vec![FolTreeNode::mk_variable("A_extra_0")], false),
            "A_extra_0",
            Quantifier::Forall,
        );
        let prop_expected = ProcessedStatProp::mk_fol("p1", formula_expected);
        assert_eq!(processed_props[0], prop_expected);
    }

    #[test]
    /// Test processing of a FOL property that requires function expression substitutions.
    fn test_process_fol_prop_with_substitution() {
        // Create basic sketch with no regulation props, single variable `A` and fn symbol `f`
        let aeon_str = "A-??A\n$A:f(A)";
        let mut sketch = Sketch::from_aeon(aeon_str).unwrap();
        // Add expression for f
        sketch
            .model
            .set_uninterpreted_fn_expression_by_str("f", "!var0")
            .unwrap();

        // Add simple FOL property involving function f
        let prop = StatProperty::try_mk_generic("p1", "\\forall x: f(x)").unwrap();
        sketch.properties.add_static_by_str("p1", prop).unwrap();

        // Process the static properties and check results - function symbol must get replaced
        // by its expression in the formula, plus the variable's name is standardized
        let (bn, expressions_map) = InferenceSolver::extract_bn(&sketch).unwrap();
        let processed_props = process_static_props(&sketch, &bn, expressions_map, "A").unwrap();
        let formula_expected = FolTreeNode::mk_quantifier(
            FolTreeNode::mk_unary(FolTreeNode::mk_variable("A_extra_0"), UnaryOp::Not),
            "A_extra_0",
            Quantifier::Forall,
        );
        let prop_expected = ProcessedStatProp::mk_fol("p1", formula_expected);
        assert_eq!(processed_props[0], prop_expected);
    }

    #[test]
    /// Test processing of a simple template function property that does not require function
    /// expression substitutions.
    fn test_process_fn_prop_simple() {
        // Create basic sketch with no regulation props, single variable `A`
        // and fn symbol `f` (with no expression)
        let aeon_str = "A-??A\n$A:f(A)";
        let mut sketch = Sketch::from_aeon(aeon_str).unwrap();
        let fn_f = sketch.model.get_uninterpreted_fn_id("f").unwrap();

        // Add simple function property involving function f
        let prop =
            StatProperty::mk_fn_input_essential("p1", Some(0), Some(fn_f), Essentiality::True);
        sketch.properties.add_static_by_str("p1", prop).unwrap();

        // Process the static properties and check the encoded result (standard encoding)
        let (bn, expressions_map) = InferenceSolver::extract_bn(&sketch).unwrap();
        let processed_props = process_static_props(&sketch, &bn, expressions_map, "A").unwrap();
        let formula_expected = FolTreeNode::mk_binary(
            FolTreeNode::mk_function("f", vec![FolTreeNode::mk_constant(false)], false),
            FolTreeNode::mk_function("f", vec![FolTreeNode::mk_constant(true)], false),
            BinaryOp::Xor,
        );
        let prop_expected = ProcessedStatProp::mk_fol("p1", formula_expected);
        assert_eq!(processed_props[0], prop_expected);
    }

    #[test]
    /// Test processing of a simple template regulation property.
    fn test_process_reg_prop() {
        // Create basic sketch with no regulation props and single variable `A`.
        let mut sketch = Sketch::from_aeon("A-??A").unwrap();
        let var_a = sketch.model.get_var_id("A").unwrap();

        // Add simple regulation property involving A
        let prop = StatProperty::mk_regulation_essential(
            "p1",
            Some(var_a.clone()),
            Some(var_a),
            Essentiality::True,
        );
        sketch.properties.add_static_by_str("p1", prop).unwrap();

        // Process the static properties and check the encoded result (standard encoding with
        // implicit update function symbols)
        let (bn, expressions_map) = InferenceSolver::extract_bn(&sketch).unwrap();
        let processed_props = process_static_props(&sketch, &bn, expressions_map, "A").unwrap();
        let formula_expected = FolTreeNode::mk_binary(
            FolTreeNode::mk_function("f_A", vec![FolTreeNode::mk_constant(false)], true),
            FolTreeNode::mk_function("f_A", vec![FolTreeNode::mk_constant(true)], true),
            BinaryOp::Xor,
        );
        let prop_expected = ProcessedStatProp::mk_fol("p1", formula_expected);
        assert_eq!(processed_props[0], prop_expected);
    }

    #[test]
    /// Test processing of a simple template function property that requires function
    /// expression substitutions.
    fn test_process_fn_prop_with_substitution() {
        // Create basic sketch with no regulation props, single variable `A`
        // and fn symbol `f` (with no expression)
        let aeon_str = "A-??A\n$A:f(A)";
        let mut sketch = Sketch::from_aeon(aeon_str).unwrap();
        let fn_f = sketch.model.get_uninterpreted_fn_id("f").unwrap();

        // Add expression for f
        sketch
            .model
            .set_uninterpreted_fn_expression_by_str("f", "!var0")
            .unwrap();

        // Add simple function property involving function f
        let prop =
            StatProperty::mk_fn_input_essential("p1", Some(0), Some(fn_f), Essentiality::True);
        sketch.properties.add_static_by_str("p1", prop).unwrap();

        // Process the static properties and check results - function symbol in the resulting
        // FOL formula must get replaced by its expression
        let (bn, expressions_map) = InferenceSolver::extract_bn(&sketch).unwrap();
        let processed_props = process_static_props(&sketch, &bn, expressions_map, "A").unwrap();
        let formula_expected = FolTreeNode::mk_binary(
            FolTreeNode::mk_unary(FolTreeNode::mk_constant(false), UnaryOp::Not),
            FolTreeNode::mk_unary(FolTreeNode::mk_constant(true), UnaryOp::Not),
            BinaryOp::Xor,
        );
        let prop_expected = ProcessedStatProp::mk_fol("p1", formula_expected);
        assert_eq!(processed_props[0], prop_expected);
    }
}
