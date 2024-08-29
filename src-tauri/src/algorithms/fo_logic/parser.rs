use crate::algorithms::fo_logic::fol_tree::*;
use crate::algorithms::fo_logic::operator_enums::*;
use crate::algorithms::fo_logic::tokenizer::{try_tokenize_formula, FolToken};
use crate::algorithms::fo_logic::utils::validate_and_rename_vars;

/// Parse an FOL formula string representation into an actual formula tree.
/// Basically a wrapper for tokenize+parse (used often for testing/debug purposes).
///
/// NEEDS to call [validate_props] to fully finish the preprocessing step.
pub fn parse_fol_formula(formula: &str) -> Result<FolTreeNode, String> {
    let tokens = try_tokenize_formula(formula.to_string())?;
    let tree = parse_fol_tokens(&tokens)?;
    Ok(tree)
}

/// Parse an FOL formula string representation into an actual formula tree with renamed (minimized)
/// set of variables.
///
/// Basically a wrapper for the whole preprocessing step (tokenize + parse + rename vars).
///
/// The format of variable names is given by how [SymbolicContext::with_extra_state_variables]
/// creates new extra variables. Basically, we choose a name of one BN variable (`var_core_name`),
/// and it is used as a base for extra variables `{var_base_name}_extra_{index}`.
pub fn parse_and_minimize_fol_formula(
    formula: &str,
    base_var_name: &str,
) -> Result<FolTreeNode, String> {
    let tree = parse_fol_formula(formula)?;
    let tree = validate_and_rename_vars(tree, base_var_name)?;
    Ok(tree)
}

/// Predicate for whether given token represents a quantifier.
fn is_quantifier(token: &FolToken) -> bool {
    matches!(token, FolToken::Quantifier(..))
}

/// Predicate for whether given token represents unary operator.
fn is_unary(token: &FolToken) -> bool {
    matches!(token, FolToken::Unary(_))
}

/// Utility method to find the first occurrence of a specific token in the token tree.
fn index_of_first(tokens: &[FolToken], token: FolToken) -> Option<usize> {
    return tokens.iter().position(|t| *t == token);
}

/// Utility method to find the first occurrence of a quantifier operator in the token tree.
fn index_of_first_quantifier(tokens: &[FolToken]) -> Option<usize> {
    return tokens.iter().position(is_quantifier);
}

/// Utility method to find the first occurrence of an unary operator in the token tree.
fn index_of_first_unary(tokens: &[FolToken]) -> Option<usize> {
    return tokens.iter().position(is_unary);
}

/// Parse `tokens` of FOL formula into an abstract syntax tree using recursive steps.
pub fn parse_fol_tokens(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
    parse_1_quantifiers(tokens)
}

/// Recursive parsing step 1: extract quantifier operators.
///
/// Quantifier must not be immediately preceded by any other kind of operator.
/// We only allow it to be preceded by another quantifier, otherwise parentheses must be used.
/// (things like "!V x: ..." are forbidden, must be written in parentheses as "!(V x: ...)"
fn parse_1_quantifiers(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
    let quantifier_token = index_of_first_quantifier(tokens);
    Ok(if let Some(i) = quantifier_token {
        // perform check that hybrid operator is not preceded by other type of operators
        if i > 0 && !matches!(&tokens[i - 1], FolToken::Quantifier(..)) {
            return Err(format!(
                "Quantifier can't be directly preceded by '{}'.",
                &tokens[i - 1]
            ));
        }
        match &tokens[i] {
            FolToken::Quantifier(op, var) => FolTreeNode::mk_quantifier(
                parse_1_quantifiers(&tokens[(i + 1)..])?,
                var.as_str(),
                op.clone(),
            ),
            _ => unreachable!(), // we already made sure that this is indeed a quantifier token
        }
    } else {
        parse_2_iff(tokens)?
    })
}

/// Recursive parsing step 2: extract `<=>` operators.
fn parse_2_iff(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
    let iff_token = index_of_first(tokens, FolToken::Binary(BinaryOp::Iff));
    Ok(if let Some(i) = iff_token {
        FolTreeNode::mk_binary(
            parse_3_imp(&tokens[..i])?,
            parse_2_iff(&tokens[(i + 1)..])?,
            BinaryOp::Iff,
        )
    } else {
        parse_3_imp(tokens)?
    })
}

/// Recursive parsing step 3: extract `=>` operators.
fn parse_3_imp(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
    let imp_token = index_of_first(tokens, FolToken::Binary(BinaryOp::Imp));
    Ok(if let Some(i) = imp_token {
        FolTreeNode::mk_binary(
            parse_4_or(&tokens[..i])?,
            parse_3_imp(&tokens[(i + 1)..])?,
            BinaryOp::Imp,
        )
    } else {
        parse_4_or(tokens)?
    })
}

/// Recursive parsing step 4: extract `|` operators.
fn parse_4_or(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
    let or_token = index_of_first(tokens, FolToken::Binary(BinaryOp::Or));
    Ok(if let Some(i) = or_token {
        FolTreeNode::mk_binary(
            parse_5_xor(&tokens[..i])?,
            parse_4_or(&tokens[(i + 1)..])?,
            BinaryOp::Or,
        )
    } else {
        parse_5_xor(tokens)?
    })
}

/// Recursive parsing step 5: extract `^` operators.
fn parse_5_xor(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
    let xor_token = index_of_first(tokens, FolToken::Binary(BinaryOp::Xor));
    Ok(if let Some(i) = xor_token {
        FolTreeNode::mk_binary(
            parse_6_and(&tokens[..i])?,
            parse_5_xor(&tokens[(i + 1)..])?,
            BinaryOp::Xor,
        )
    } else {
        parse_6_and(tokens)?
    })
}

/// Recursive parsing step 6: extract `&` operators.
fn parse_6_and(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
    let and_token = index_of_first(tokens, FolToken::Binary(BinaryOp::And));
    Ok(if let Some(i) = and_token {
        FolTreeNode::mk_binary(
            parse_7_unary(&tokens[..i])?,
            parse_6_and(&tokens[(i + 1)..])?,
            BinaryOp::And,
        )
    } else {
        parse_7_unary(tokens)?
    })
}

/// Recursive parsing step 7: extract unary operators (just a negation currently).
fn parse_7_unary(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
    let unary_token = index_of_first_unary(tokens);
    Ok(if let Some(i) = unary_token {
        // perform check that unary operator is not directly preceded by some atomic sub-formula
        if i > 0 && matches!(&tokens[i - 1], FolToken::Atomic(..)) {
            return Err(format!(
                "Unary operator can't be directly preceded by '{:?}'.",
                &tokens[i - 1]
            ));
        }

        match &tokens[i] {
            FolToken::Unary(op) => {
                FolTreeNode::mk_unary(parse_7_unary(&tokens[(i + 1)..])?, op.clone())
            }
            _ => unreachable!(), // we already made sure that this is indeed an unary token
        }
    } else {
        parse_8_terms_and_parentheses(tokens)?
    })
}

/// Recursive parsing step 8: extract terms and recursively solve sub-formulae in parentheses and in
/// functions.
fn parse_8_terms_and_parentheses(tokens: &[FolToken]) -> Result<FolTreeNode, String> {
    if tokens.is_empty() {
        Err("Expected formula, found nothing.".to_string())
    } else {
        if tokens.len() == 1 {
            // This should be name (var/function) or a parenthesis group, anything
            // else does not make sense (constants are tokenized as variables until now).
            match &tokens[0] {
                FolToken::Atomic(Atom::Var(name)) => {
                    return Ok(FolTreeNode::mk_variable(name.as_str()));
                }
                FolToken::Atomic(Atom::True) => {
                    return Ok(FolTreeNode::mk_constant(true));
                }
                FolToken::Atomic(Atom::False) => {
                    return Ok(FolTreeNode::mk_constant(false));
                }
                FolToken::Function(FunctionSymbol(name), arguments) => {
                    let mut arg_expression_nodes = Vec::new();
                    for inner in arguments {
                        // it must be a token list
                        if let FolToken::TokenList(inner_token_list) = inner {
                            arg_expression_nodes.push(parse_fol_tokens(inner_token_list)?);
                        } else {
                            return Err("Function must be applied on `FolToken::TokenList` args."
                                .to_string());
                        }
                    }
                    return Ok(FolTreeNode::mk_function(name, arg_expression_nodes));
                }
                // recursively solve sub-formulae in parentheses
                FolToken::TokenList(inner) => {
                    return parse_fol_tokens(inner);
                }
                _ => {} // otherwise, fall through to the error at the end.
            }
        }
        Err(format!("Unexpected: {tokens:?}. Expecting formula."))
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithms::fo_logic::fol_tree::*;
    use crate::algorithms::fo_logic::operator_enums::*;
    use crate::algorithms::fo_logic::parser::parse_fol_formula;

    #[test]
    /// Test whether several valid FOL formulae are parsed without causing errors.
    /// Also check that the formula is saved correctly in the tree root.
    fn parse_valid_formulae() {
        let valid1 = "(\\exists x: f(x))";
        let tree = parse_fol_formula(valid1).unwrap();
        assert_eq!(tree.as_str(), valid1);
    }

    #[test]
    fn operator_priority() {
        assert_eq!(
            "(((((!a) ^ ((!b) & (!c))) | (!d)) => (!e)) <=> (!f))",
            parse_fol_formula("!a ^ !b & !c | !d => !e <=> !f")
                .unwrap()
                .as_str()
        )
    }

    #[test]
    fn operator_associativity() {
        assert_eq!(
            "(a & (b & c))",
            parse_fol_formula("a & b & c").unwrap().as_str()
        );
        assert_eq!(
            "(a | (b | c))",
            parse_fol_formula("a | b | c").unwrap().as_str()
        );
        assert_eq!(
            "(a ^ (b ^ c))",
            parse_fol_formula("a ^ b ^ c").unwrap().as_str()
        );
        assert_eq!(
            "(a => (b => c))",
            parse_fol_formula("a => b => c").unwrap().as_str()
        );
        assert_eq!(
            "(a <=> (b <=> c))",
            parse_fol_formula("a <=> b <=> c").unwrap().as_str()
        );
    }

    #[test]
    /// Test parsing of several valid FOL formulae against expected results.
    fn compare_parser_with_expected() {
        let formula = "(false & v1)";
        let expected_tree = FolTreeNode::mk_binary(
            FolTreeNode::mk_constant(false),
            FolTreeNode::mk_variable("v1"),
            BinaryOp::And,
        );
        assert_eq!(parse_fol_formula(formula).unwrap(), expected_tree);

        let formula = "\\exists x: f(x)";
        let expected_tree = FolTreeNode::mk_quantifier(
            FolTreeNode::mk_function("f", vec![FolTreeNode::mk_variable("x")]),
            "x",
            Quantifier::Exists,
        );
        assert_eq!(parse_fol_formula(formula).unwrap(), expected_tree);

        let formula = "\\forall x: \\exists yy: (f(1, !yy) & x)";
        let expected_tree = FolTreeNode::mk_quantifier(
            FolTreeNode::mk_quantifier(
                FolTreeNode::mk_binary(
                    FolTreeNode::mk_function(
                        "f",
                        vec![
                            FolTreeNode::mk_constant(true),
                            FolTreeNode::mk_unary(FolTreeNode::mk_variable("yy"), UnaryOp::Not),
                        ],
                    ),
                    FolTreeNode::mk_variable("x"),
                    BinaryOp::And,
                ),
                "yy",
                Quantifier::Exists,
            ),
            "x",
            Quantifier::Forall,
        );
        assert_eq!(parse_fol_formula(formula).unwrap(), expected_tree);
    }

    #[test]
    /// Test parsing of several completely invalid FOL formulae.
    fn parse_invalid_formulae() {
        let invalid_formulae = vec![
            "3 x: x x",
            "& x",
            "x x",
            "",
            "! \\exists x: x",
            "\\exists &: x",
        ];

        for formula in invalid_formulae {
            assert!(parse_fol_formula(formula).is_err());
        }
    }
}
