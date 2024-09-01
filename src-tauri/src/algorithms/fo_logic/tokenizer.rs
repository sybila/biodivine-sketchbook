use crate::algorithms::fo_logic::operator_enums::*;

use crate::algorithms::fo_logic::operator_enums::FunctionSymbol;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

use super::utils::is_update_fn_symbol;

/// Enum of all possible tokens occurring in a FOL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FolToken {
    Unary(UnaryOp),
    Binary(BinaryOp),
    Quantifier(Quantifier, String),
    Atomic(Atom),
    Function(FunctionSymbol, Vec<FolToken>),
    TokenList(Vec<FolToken>),
}

/// Try to tokenize given FOL formula string.
///
/// This is a wrapper for the (more general) recursive [try_tokenize_recursive]` function.
pub fn try_tokenize_formula(formula: String) -> Result<Vec<FolToken>, String> {
    let (tokens, _) = try_tokenize_recursive(&mut formula.chars().peekable(), true, false)?;
    Ok(tokens)
}

/// Process a peekable iterator of characters into a vector of `FolToken`s. This function is used
/// for both tokenizing a top-level formula an expression that is given as fn's argument.
///
/// Returns a vector of (nested) tokens, and a last character. The last character is important when
/// we are parsing function arguments (to find out if another argument is expected or we already
/// processed the closing parenthesis). When parsing the top-level formula expression (not a function
/// argument), we simply return '$'.
///
/// `top_fn_level` is used in case we are processing an expression passed as argument to some
/// function symbol (then ',' is valid delimiter).
fn try_tokenize_recursive(
    input_chars: &mut Peekable<Chars>,
    top_level: bool,
    top_fn_level: bool,
) -> Result<(Vec<FolToken>, char), String> {
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
                let var_name = collect_var_from_operator(input_chars, "3")?;
                output.push(FolToken::Quantifier(Quantifier::Exists, var_name));
            }
            // "V" can be either short for forall quantifier or part of some name
            'V' if !is_valid_in_name_optional(input_chars.peek()) => {
                let var_name = collect_var_from_operator(input_chars, "V")?;
                output.push(FolToken::Quantifier(Quantifier::Forall, var_name));
            }
            ')' => {
                return if !top_level {
                    Ok((output, ')'))
                } else {
                    Err("Unexpected ')' without opening counterpart.".to_string())
                }
            }
            '(' => {
                // start a nested token group
                let (token_group, _) = try_tokenize_recursive(input_chars, false, false)?;
                output.push(FolToken::TokenList(token_group));
            }
            // long name for quantifiers (\exists, \forall)
            '\\' => {
                // collect rest of the operator
                let quantifier_name = collect_name(input_chars)?;
                if &quantifier_name == "exists" {
                    let var_name = collect_var_from_operator(input_chars, "\\exists")?;
                    output.push(FolToken::Quantifier(Quantifier::Exists, var_name));
                } else if quantifier_name == "forall" {
                    let var_name = collect_var_from_operator(input_chars, "\\forall")?;
                    output.push(FolToken::Quantifier(Quantifier::Forall, var_name));
                } else {
                    return Err(format!("Invalid quantifier `\\{quantifier_name}`."));
                }
            }
            ',' if top_fn_level => {
                // in case we are collecting something inside a function, a comma is valid
                // delimiter
                return Ok((output, ','));
            }
            // function symbol, variable, or a constant
            c if is_valid_in_name(c) => {
                // collect full name
                let name = collect_name(input_chars)?;
                let full_name = c.to_string() + &name;
                // skip whitespaces that can appear between potential function symbol and "("
                skip_whitespaces(input_chars);

                if Some(&'(') == input_chars.peek() {
                    // it must be a function symbol with arguments in parentheses
                    let arguments = collect_fn_arguments(input_chars)?;
                    // we must check if it is symbol for update or uninterpreted fn
                    let is_update = is_update_fn_symbol(&full_name);
                    output.push(FolToken::Function(
                        FunctionSymbol::new(&full_name, is_update),
                        arguments,
                    ));
                } else {
                    // otherwise it is a variable or a constant
                    output.push(FolToken::Atomic(resolve_term_name(&full_name)));
                }
            }
            _ => return Err(format!("Unexpected char '{c}'.")),
        }
    }

    if top_level {
        Ok((output, '$'))
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

/// Predicate to decide if a given "name" represents `true` constant.
fn is_true_const(name: &str) -> bool {
    name == "true" || name == "True" || name == "TRUE" || name == "1"
}

/// Predicate to decide if a given "name" represents `false` constant.
fn is_false_const(name: &str) -> bool {
    name == "false" || name == "False" || name == "FALSE" || name == "0"
}

/// Decide whether the name corresponds to a constant or a variable, and return the
/// correct term token.
fn resolve_term_name(name: &str) -> Atom {
    if is_false_const(name) {
        Atom::False
    } else if is_true_const(name) {
        Atom::True
    } else {
        Atom::Var(name.to_string())
    }
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

/// Retrieve the name of the variable bound by a quantifier.
/// Operator string is consumed by caller and is given as input for error msg purposes.
fn collect_var_from_operator(
    input_chars: &mut Peekable<Chars>,
    operator: &str,
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

/// Retrieve the arguments of a function, process everything from "(" up to ")".
/// Function name is consumed by caller.
fn collect_fn_arguments(input_chars: &mut Peekable<Chars>) -> Result<Vec<FolToken>, String> {
    input_chars.next(); // skip the "("

    // check special case when we dont have any arguments (constant parameter, e.g., "P()")
    if let Some(&')') = input_chars.peek() {
        input_chars.next(); // skip the ")"
        return Ok(Vec::new());
    }

    let mut fn_args = Vec::new();
    let mut last_delim_char = ',';
    // iterate until ")" is processed by a sub-function
    while last_delim_char != ')' {
        // delimiters must be always "," until we reach ")" and break from the loop
        assert_eq!(last_delim_char, ',');

        let (token_group, last_char) = try_tokenize_recursive(input_chars, false, true)?;
        if token_group.is_empty() {
            return Err("Function argument can't be empty.".to_string());
        }
        fn_args.push(FolToken::TokenList(token_group));
        last_delim_char = last_char;
    }

    Ok(fn_args)
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
            FolToken::Atomic(Atom::Var(name)) => write!(f, "{name}"),
            FolToken::Atomic(constant) => write!(f, "{constant:?}"),
            FolToken::Function(name, _) => write!(f, "{name}(...)"),
            FolToken::TokenList(_) => write!(f, "TokenList"), // debug purposes only
        }
    }
}

/// Recursively display tokens.
fn print_tokens_recursively(tokens: &Vec<FolToken>) {
    for token in tokens {
        match token {
            FolToken::Unary(UnaryOp::Not) => print!("!"),
            FolToken::Binary(BinaryOp::And) => print!("&"),
            FolToken::Binary(BinaryOp::Or) => print!("|"),
            FolToken::Binary(BinaryOp::Xor) => print!("^"),
            FolToken::Binary(BinaryOp::Imp) => print!("=>"),
            FolToken::Binary(BinaryOp::Iff) => print!("<=>"),
            FolToken::Quantifier(op, var) => print!("{op:?} {var}:"),
            FolToken::Atomic(Atom::Var(name)) => print!("{name}"),
            FolToken::Atomic(constant) => print!("{constant:?}"),
            FolToken::TokenList(token_vec) => print_tokens_recursively(token_vec),
            FolToken::Function(name, args) => {
                print!("{name}(");
                for arg in args {
                    print_tokens_recursively(&vec![arg.clone()]);
                    // todo: one more "," than needed
                    print!(",")
                }
                print!(")")
            }
        }
    }
}

/// Print the vector of tokens (for debug purposes).
pub fn print_tokens(tokens: &Vec<FolToken>) {
    print_tokens_recursively(tokens);
    println!();
}

#[cfg(test)]
mod tests {
    use crate::algorithms::fo_logic::operator_enums::*;
    use crate::algorithms::fo_logic::tokenizer::{try_tokenize_formula, FolToken};
    use std::vec;

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
                FolToken::Function(
                    FunctionSymbol::new_uninterpreted("f"),
                    vec![FolToken::TokenList(vec![FolToken::Atomic(Atom::Var(
                        "x".to_string()
                    ))])],
                ),
            ]
        );

        let valid2 = "\\forall x: \\exists yy: f(x, !yy)".to_string();
        let tokens2 = try_tokenize_formula(valid2).unwrap();
        assert_eq!(
            tokens2,
            vec![
                FolToken::Quantifier(Quantifier::Forall, "x".to_string()),
                FolToken::Quantifier(Quantifier::Exists, "yy".to_string()),
                FolToken::Function(
                    FunctionSymbol::new_uninterpreted("f"),
                    vec![
                        FolToken::TokenList(vec![FolToken::Atomic(Atom::Var("x".to_string()))]),
                        FolToken::TokenList(vec![
                            FolToken::Unary(UnaryOp::Not),
                            FolToken::Atomic(Atom::Var("yy".to_string())),
                        ]),
                    ],
                ),
            ]
        );

        let valid3 = "fn()".to_string();
        let tokens3 = try_tokenize_formula(valid3).unwrap();
        assert_eq!(
            tokens3,
            vec![FolToken::Function(
                FunctionSymbol::new_uninterpreted("fn"),
                vec![],
            ),]
        );
    }

    #[test]
    /// Test tokenization process on FOL formula with several whitespaces.
    fn tokenize_with_whitespaces() {
        let valid_formula = " 3   x  :  f    ( x ,  y )  ";
        assert!(try_tokenize_formula(valid_formula.to_string()).is_ok())
    }

    #[test]
    /// Test tokenization process on several invalid FOL formulae.
    /// Try to cover wide range of invalid possibilities, as well as potential frequent mistakes.
    fn tokenize_invalid_formulae() {
        let invalid_formulae = vec![
            "x1 )", "( x1", "x1 <> x2", "x1 >= x2", "x1 <= x2", "\\ex x", "\\fora x", "f(x,)",
            "f(x,", "f(x", "f(x x))",
        ];

        for formula in invalid_formulae {
            assert!(try_tokenize_formula(formula.to_string()).is_err())
        }
    }
}
