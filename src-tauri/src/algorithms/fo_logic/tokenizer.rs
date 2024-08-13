use crate::algorithms::fo_logic::operator_enums::*;

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

/// Enum of all possible tokens occurring in a FOL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FolToken {
    Unary(UnaryOp),
    Binary(BinaryOp),
    Quantifier(Quantifier, String),
    Term(Term),
    Tokens(Vec<FolToken>),
}

/// Try to tokenize given FOL formula string.
///
/// This is a wrapper for the (more general) recursive [try_tokenize_recursive]` function.
pub fn try_tokenize_formula(formula: String) -> Result<Vec<FolToken>, String> {
    try_tokenize_recursive(&mut formula.chars().peekable(), true)
}

/// Process a peekable iterator of characters into a vector of `FolToken`s.
fn try_tokenize_recursive(
    input_chars: &mut Peekable<Chars>,
    top_level: bool,
) -> Result<Vec<FolToken>, String> {
    let mut output = Vec::new();

    while let Some(c) = input_chars.next() {
        match c {
            c if c.is_whitespace() => {} // skip whitespace
            '!' => output.push(FolToken::Unary(UnaryOp::Not)),
            '&' => output.push(FolToken::Binary(BinaryOp::And)),
            '|' => output.push(FolToken::Binary(BinaryOp::Or)),
            '^' => output.push(FolToken::Binary(BinaryOp::Xor)),
            '=' => {
                if Some('>') == input_chars.next() {
                    output.push(FolToken::Binary(BinaryOp::Imp));
                } else {
                    return Err("Expected '>' after '='.".to_string());
                }
            }
            '<' => {
                if Some('=') == input_chars.next() {
                    if Some('>') == input_chars.next() {
                        output.push(FolToken::Binary(BinaryOp::Iff));
                    } else {
                        return Err("Expected '>' after '<='.".to_string());
                    }
                } else {
                    return Err("Expected '=' after '<'.".to_string());
                }
            }
            // '>' is invalid as a start of a token
            '>' => return Err("Unexpected '>'.".to_string()),

            // "3" can be either short for exist quantifier or part of some name
            '3' if !is_valid_in_name_optional(input_chars.peek()) => {
                let var_name = collect_var_from_operator(input_chars, '3')?;
                output.push(FolToken::Quantifier(Quantifier::Exists, var_name));
            }
            // "V" can be either short for forall quantifier or part of some name
            'V' if !is_valid_in_name_optional(input_chars.peek()) => {
                let var_name = collect_var_from_operator(input_chars, 'V')?;
                output.push(FolToken::Quantifier(Quantifier::Forall, var_name));
            }
            ')' => {
                return if !top_level {
                    Ok(output)
                } else {
                    Err("Unexpected ')' without opening counterpart.".to_string())
                }
            }
            '(' => {
                // start a nested token group
                let token_group = try_tokenize_recursive(input_chars, false)?;
                output.push(FolToken::Tokens(token_group));
            }
            // long name for quantifiers (\exists, \forall)
            '\\' => {
                // collect rest of the operator
                let quantifier_name = collect_name(input_chars)?;
                if &quantifier_name == "exists" {
                    let var_name = collect_var_from_operator(input_chars, '3')?;
                    output.push(FolToken::Quantifier(Quantifier::Exists, var_name));
                } else if quantifier_name == "forall" {
                    let var_name = collect_var_from_operator(input_chars, 'V')?;
                    output.push(FolToken::Quantifier(Quantifier::Forall, var_name));
                } else {
                    return Err(format!("Invalid quantifier `\\{quantifier_name}`."));
                }
            }
            // function symbol, variable, or a constant
            // these 2 are NOT distinguished now but later during parsing
            c if is_valid_in_name(c) => {
                // collect full name
                let name = collect_name(input_chars)?;
                // skip whitespaces that can appear between potential function symbol and "("
                skip_whitespaces(input_chars);

                if Some(&'(') == input_chars.peek() {
                    // it must be a function symbol with arguments in parentheses
                    let mut fn_args = Vec::new();
                    input_chars.next(); // skip the "("
                    while Some(&')') != input_chars.peek() {
                        skip_whitespaces(input_chars);

                        // arguments can be negated
                        let mut negated = false;
                        if Some(&'!') == input_chars.peek() {
                            skip_whitespaces(input_chars);
                            negated = true;
                            input_chars.next(); // skip the "!"
                        }
                        skip_whitespaces(input_chars);

                        let arg_name = collect_name(input_chars)?;
                        if arg_name.is_empty() {
                            return Err("Variable name can't be empty.".to_string());
                        }
                        fn_args.push((negated, Term::Var(arg_name)));
                        skip_whitespaces(input_chars);

                        // next must be either "," or ")"
                        if Some(&')') == input_chars.peek() {
                            continue;
                        } else if Some(',') != input_chars.next() {
                            return Err(format!(
                                "Expected ',' after function's argument (argument of fn '{}').",
                                c.to_string() + &name
                            ));
                        }
                    }
                    input_chars.next(); // skip the ")"
                    output.push(FolToken::Term(Term::Function(
                        c.to_string() + &name,
                        fn_args,
                    )));
                } else {
                    // otherwise it is a variable or constant
                    // we will save it as a variable for now, and convert later in parser if needed
                    output.push(FolToken::Term(Term::Var(c.to_string() + &name)));
                }
            }
            _ => return Err(format!("Unexpected char '{c}'.")),
        }
    }

    if top_level {
        Ok(output)
    } else {
        Err("Expected ')' to previously encountered opening counterpart.".to_string())
    }
}

/// Check all whitespaces at the front of the iterator.
fn skip_whitespaces(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next(); // Skip the whitespace character
        } else {
            break; // Stop skipping when a non-whitespace character is found
        }
    }
}

/// Check if given char can appear in a name.
fn is_valid_in_name(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Check if given char can appear in a name.
fn is_valid_in_name_optional(option_char: Option<&char>) -> bool {
    if let Some(c) = option_char {
        return is_valid_in_name(*c);
    }
    false
}

/// Retrieve the name (of a proposition or variable) from the input.
/// The first character of the name may or may not be already consumed by the caller.
fn collect_name(input_chars: &mut Peekable<Chars>) -> Result<String, String> {
    let mut name = Vec::new();
    while let Some(c) = input_chars.peek() {
        if !is_valid_in_name(*c) {
            break;
        } else {
            name.push(*c);
            input_chars.next(); // advance iterator
        }
    }
    Ok(name.into_iter().collect())
}

/// Retrieve the name of the variable, and optional name for the domain, bound by a hybrid operator.
/// Operator character is consumed by caller and is given as input for error msg purposes.
fn collect_var_from_operator(
    input_chars: &mut Peekable<Chars>,
    operator: char,
) -> Result<String, String> {
    // there might be few spaces first
    skip_whitespaces(input_chars);
    // now collect the variable name itself
    let name = collect_name(input_chars)?;
    if name.is_empty() {
        return Err("Variable name can't be empty.".to_string());
    }
    skip_whitespaces(input_chars);

    if Some(':') != input_chars.next() {
        return Err(format!(
            "Expected ':' after segment of quantifier '{operator}'."
        ));
    }
    Ok(name)
}

impl fmt::Display for FolToken {
    /// Display tokens for debug purposes.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FolToken::Unary(UnaryOp::Not) => write!(f, "!"),
            FolToken::Binary(BinaryOp::And) => write!(f, "&"),
            FolToken::Binary(BinaryOp::Or) => write!(f, "|"),
            FolToken::Binary(BinaryOp::Xor) => write!(f, "^"),
            FolToken::Binary(BinaryOp::Imp) => write!(f, "=>"),
            FolToken::Binary(BinaryOp::Iff) => write!(f, "<=>"),
            FolToken::Quantifier(op, var) => write!(f, "{op:?} {var}:"),
            FolToken::Term(Term::Var(name)) => write!(f, "{name}"),
            FolToken::Term(Term::Function(name, args)) => write!(f, "{name}({args:?})"),
            FolToken::Term(constant) => write!(f, "{constant:?}"),
            FolToken::Tokens(_) => write!(f, "( TOKENS )"), // debug purposes only
        }
    }
}

#[allow(dead_code)]
/// Recursively print tokens.
fn print_tokens_recursively(tokens: &Vec<FolToken>) {
    for token in tokens {
        match token {
            FolToken::Tokens(token_vec) => print_tokens_recursively(token_vec),
            _ => print!("{token} "),
        }
    }
}

#[allow(dead_code)]
/// Print the vector of tokens (for debug purposes).
pub fn print_tokens(tokens: &Vec<FolToken>) {
    print_tokens_recursively(tokens);
    println!();
}

#[cfg(test)]
mod tests {
    use crate::algorithms::fo_logic::operator_enums::*;
    use crate::algorithms::fo_logic::tokenizer::{try_tokenize_formula, FolToken};

    #[test]
    /// Test tokenization process on several valid FOL formulae.
    /// Test both some important and meaningful formulae and formulae that include wide
    /// range of operators.
    fn tokenize_valid_formulae() {
        let valid1 = "3 x: f(x)".to_string();
        let tokens1 = try_tokenize_formula(valid1).unwrap();
        assert_eq!(
            tokens1,
            vec![
                FolToken::Quantifier(Quantifier::Exists, "x".to_string()),
                FolToken::Term(Term::Function(
                    "f".to_string(),
                    vec![(false, Term::Var("x".to_string()))]
                )),
            ]
        );

        let valid2 = "\\forall x: \\exists yy: f(x, !yy)".to_string();
        let tokens2 = try_tokenize_formula(valid2).unwrap();
        assert_eq!(
            tokens2,
            vec![
                FolToken::Quantifier(Quantifier::Forall, "x".to_string()),
                FolToken::Quantifier(Quantifier::Exists, "yy".to_string()),
                FolToken::Term(Term::Function(
                    "f".to_string(),
                    vec![
                        (false, Term::Var("x".to_string())),
                        (true, Term::Var("yy".to_string()))
                    ]
                )),
            ]
        );
    }

    #[test]
    /// Test tokenization process on FOL formula with several whitespaces.
    fn tokenize_with_whitespaces() {
        let valid_formula = " 3     x     :  f    ( x  )  ";
        assert!(try_tokenize_formula(valid_formula.to_string()).is_ok())
    }

    #[test]
    /// Test tokenization process on several invalid FOL formulae.
    /// Try to cover wide range of invalid possibilities, as well as potential frequent mistakes.
    fn tokenize_invalid_formulae() {
        let invalid_formulae = vec!["x1 )", "( x1", "x1 <> x2", "x1 >= x2", "x1 <= x2"];

        for formula in invalid_formulae {
            assert!(try_tokenize_formula(formula.to_string()).is_err())
        }
    }
}
