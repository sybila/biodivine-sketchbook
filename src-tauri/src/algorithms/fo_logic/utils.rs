use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, SymbolicContext};
use biodivine_lib_param_bn::BooleanNetwork;
use regex::Regex;

/// Internally, all FOL variables are encoded using an arbitrary BN variable name, and
/// an offset. For example, if `x` is chosen as a base, all FOL variables are transformed
/// to `x_extra_0`, `x_extra_1`, ...
///
/// For a given internal FOL variable name, this method returns the base BN variable and
/// the offset that was used to add it to the symbolic context. So, given `x_extra_0`, it
/// returns `x` and `0`. Error is returned if the var name format is wrong.
pub fn get_var_base_and_offset(var_name: &str) -> Result<(String, usize), String> {
    // we must get the correct "extra" BDD variable from the name of the variable
    let re = Regex::new(r"^(?P<network_variable>.+)_extra_(?P<i>\d+)$").unwrap();
    if let Some(captures) = re.captures(var_name) {
        let base_var_name = captures.name("network_variable").unwrap().as_str();
        let offset: usize = captures.name("i").unwrap().as_str().parse().unwrap();
        Ok((base_var_name.to_string(), offset))
    } else {
        Err(format!(
            "The FOL variable name string `{var_name}` did not match the expected format."
        ))
    }
}

/// If the provided function symbol corresponds to (implicit) update function for some
/// variable, get the variable's name.
/// Return Err if the symbol is not in format "f_VAR".
///
/// Note that function symbols can be either for (explicit) uninterpreted functions or
/// for (implicit) update functions. The update function symbol for variable A must be
/// in a form of "f_A".
///
/// Always expects valid `fn_symbol` name on input.
pub fn get_var_from_implicit(fn_symbol: &str) -> Result<String, String> {
    let re = Regex::new(r"^f_(?P<network_variable>.+)$").unwrap();
    if let Some(captures) = re.captures(fn_symbol) {
        let var_name = captures.name("network_variable").unwrap().as_str();
        Ok(var_name.to_string())
    } else {
        Err(format!(
            " `{fn_symbol}` is not valid symbol for an update function."
        ))
    }
}

/// Check if a given function symbol name corresponds to an (implicit) update function.
///
/// Note that function symbols can be either for (explicit) uninterpreted functions or
/// for (implicit) update functions. The update function symbol for variable A must be
/// in a form of "f_A".
///
/// Always expects valid `fn_symbol` name on input.
pub fn is_update_fn_symbol(fn_symbol: &str) -> bool {
    // this checks the format (if it is Ok it's update fn; if it is Err it's uninterpreted)
    get_var_from_implicit(fn_symbol).is_ok()
}

/// For a given network variable, create a name for its "anonymous update function".
///
/// Internally, each variable gets a function symbol to represent its update function.
/// For example, to reason about update fn of variable `A`, we use `f_A`.
pub fn get_implicit_function_name(variable_name: &str) -> String {
    format!("f_{}", variable_name)
}

/// Check that the BDD represenation for the given extended symbolic graph
/// supports the given extra variable.
pub fn check_fol_var_support(graph: &SymbolicAsyncGraph, var_name: &str) -> bool {
    if let Ok((base_var_name, offset)) = get_var_base_and_offset(var_name) {
        if let Some(base_var) = graph
            .as_network()
            .unwrap()
            .as_graph()
            .find_variable(&base_var_name)
        {
            let num_extra = graph
                .symbolic_context()
                .extra_state_variables(base_var)
                .len();
            return offset < num_extra;
        }
        return false;
    }
    false
}

/// Check that symbolic context supports the given function symbol (parameter) of given arity.
pub fn check_fn_symbol_support(ctx: &SymbolicContext, fn_name: &str, arity: usize) -> bool {
    if let Some(param) = ctx.find_network_parameter(fn_name) {
        arity == ctx.get_network_parameter_arity(param) as usize
    } else {
        false
    }
}

/// Check that the given variable is valid in the given BN, and that it has the
/// correct number of regulators.
pub fn check_update_fn_support(bn: &BooleanNetwork, var_name: &str, num_regulators: usize) -> bool {
    if let Some(var) = bn.as_graph().find_variable(var_name) {
        num_regulators == bn.regulators(var).len()
    } else {
        false
    }
}
