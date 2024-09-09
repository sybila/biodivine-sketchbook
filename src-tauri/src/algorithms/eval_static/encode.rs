use crate::algorithms::fo_logic::utils::get_implicit_function_name;
use crate::sketchbook::model::Essentiality;
use crate::sketchbook::model::Monotonicity;

use biodivine_lib_param_bn::BooleanNetwork;

/// TODO: add encoding for dual regulations
pub fn encode_regulation_monotonicity(
    input: &str,
    target: &str,
    monotonicity: Monotonicity,
    bn: &BooleanNetwork,
) -> String {
    if let Monotonicity::Unknown = monotonicity {
        return "true".to_string();
    }

    let target_var = bn.as_graph().find_variable(target).unwrap();
    let input_var = bn.as_graph().find_variable(input).unwrap();
    let regulators = bn.regulators(target_var);

    let number_inputs = regulators.len();
    let index = regulators.iter().position(|var| *var == input_var).unwrap();

    let fn_name = get_implicit_function_name(target);

    let mut quantifier_args = String::new();
    let mut left_fn_args = String::new();
    let mut right_fn_args = String::new();
    for i in 0..number_inputs {
        if i == index {
            match monotonicity {
                Monotonicity::Activation => {
                    left_fn_args.push_str("0, ");
                    right_fn_args.push_str("1, ");
                }
                Monotonicity::Inhibition => {
                    left_fn_args.push_str("1, ");
                    right_fn_args.push_str("0, ");
                }
                // TODO: cant yet deal with Dual regulations
                _ => todo!(),
            }
        } else {
            quantifier_args.push_str(format!("x_{i}, ").as_str());
            left_fn_args.push_str(format!("x_{i}, ").as_str());
            right_fn_args.push_str(format!("x_{i}, ").as_str());
        }
    }
    left_fn_args = left_fn_args.strip_suffix(", ").unwrap().to_string();
    right_fn_args = right_fn_args.strip_suffix(", ").unwrap().to_string();

    if number_inputs > 1 {
        quantifier_args = quantifier_args.strip_suffix(", ").unwrap().to_string();
        format!(
            "\\forall {quantifier_args}: {fn_name}({left_fn_args}) => {fn_name}({right_fn_args})"
        )
    } else {
        // no quantified variables
        format!("{fn_name}({left_fn_args}) => {fn_name}({right_fn_args})")
    }
}

pub fn encode_regulation_essentiality(
    input: &str,
    target: &str,
    essentiality: Essentiality,
    bn: &BooleanNetwork,
) -> String {
    if let Essentiality::Unknown = essentiality {
        return "true".to_string();
    }

    let target_var = bn.as_graph().find_variable(target).unwrap();
    let input_var = bn.as_graph().find_variable(input).unwrap();
    let regulators = bn.regulators(target_var);

    let number_inputs = regulators.len();
    let index = regulators.iter().position(|var| *var == input_var).unwrap();

    let fn_name = get_implicit_function_name(target);

    let mut quantifier_args = String::new();
    let mut left_fn_args = String::new();
    let mut right_fn_args = String::new();
    for i in 0..number_inputs {
        if i == index {
            left_fn_args.push_str("0, ");
            right_fn_args.push_str("1, ");
        } else {
            quantifier_args.push_str(format!("x_{i}, ").as_str());
            left_fn_args.push_str(format!("x_{i}, ").as_str());
            right_fn_args.push_str(format!("x_{i}, ").as_str());
        }
    }
    left_fn_args = left_fn_args.strip_suffix(", ").unwrap().to_string();
    right_fn_args = right_fn_args.strip_suffix(", ").unwrap().to_string();

    let formula = if number_inputs > 1 {
        quantifier_args = quantifier_args.strip_suffix(", ").unwrap().to_string();
        format!(
            "\\exists {quantifier_args}: {fn_name}({left_fn_args}) ^ {fn_name}({right_fn_args})"
        )
    } else {
        // no quantified variables
        format!("{fn_name}({left_fn_args}) ^ {fn_name}({right_fn_args})")
    };

    match essentiality {
        Essentiality::True => formula,
        Essentiality::False => format!("!({formula})"),
        Essentiality::Unknown => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Test encoding of monotonicity for regulations.
    fn test_encoding_monotonicity() {
        // dummy model, just to have list of variables and numbers of regulators
        let aeon_str = r#"
        A -? B
        B -> C
        A -> C
        C -| C
        "#;
        let bn = BooleanNetwork::try_from(aeon_str).unwrap();

        // encode that regulation B -> C is positive
        let fol_formula = encode_regulation_monotonicity("B", "C", Monotonicity::Activation, &bn);
        let expected = "\\forall x_0, x_2: f_C(x_0, 0, x_2) => f_C(x_0, 1, x_2)";
        assert_eq!(&fol_formula, expected);

        // encode that regulation C -| C is negative
        let fol_formula = encode_regulation_monotonicity("C", "C", Monotonicity::Inhibition, &bn);
        let expected = "\\forall x_0, x_1: f_C(x_0, x_1, 1) => f_C(x_0, x_1, 0)";
        assert_eq!(&fol_formula, expected);

        // encode that regulation A -| B is unknown
        let fol_formula = encode_regulation_monotonicity("A", "B", Monotonicity::Unknown, &bn);
        let expected = "true";
        assert_eq!(&fol_formula, expected);
    }

    #[test]
    /// Test encoding of essentiality for regulations.
    fn test_encoding_essentiality() {
        // dummy model, just to have list of variables and numbers of regulators (types of regulations dont matter)
        let aeon_str = r#"
        A ->? B
        B -> C
        A -> C
        C -| C
        "#;
        let bn = BooleanNetwork::try_from(aeon_str).unwrap();

        // encode that regulation B -> C is essential
        let fol_formula = encode_regulation_essentiality("B", "C", Essentiality::True, &bn);
        let expected = "\\exists x_0, x_2: f_C(x_0, 0, x_2) ^ f_C(x_0, 1, x_2)";
        assert_eq!(&fol_formula, expected);

        // encode that regulation C -| C has no effect (hypothetical)
        let fol_formula = encode_regulation_essentiality("C", "C", Essentiality::False, &bn);
        let expected = "!(\\exists x_0, x_1: f_C(x_0, x_1, 0) ^ f_C(x_0, x_1, 1))";
        assert_eq!(&fol_formula, expected);

        // encode that regulation A ->? B has unknown essentiality
        let fol_formula = encode_regulation_essentiality("A", "B", Essentiality::Unknown, &bn);
        let expected = "true";
        assert_eq!(&fol_formula, expected);
    }
}
