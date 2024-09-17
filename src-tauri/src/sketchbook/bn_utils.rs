use crate::sketchbook::model::{Essentiality, Monotonicity};
use biodivine_lib_param_bn::Monotonicity as Lib_Pbn_Monotonicity;

/// Utility to convert monotonicity enum used by `lib_param_bn` into the type used here.
///
/// TODO: note that `lib-param-bn` currently cannot express `Dual` variant of `Monotonicity`.
pub fn sign_from_monotonicity(monotonicity: Option<Lib_Pbn_Monotonicity>) -> Monotonicity {
    match monotonicity {
        Some(m) => match m {
            Lib_Pbn_Monotonicity::Activation => Monotonicity::Activation,
            Lib_Pbn_Monotonicity::Inhibition => Monotonicity::Inhibition,
        },
        None => Monotonicity::Unknown,
    }
}

/// Utility to convert regulation sign from enum type used in this crate into the type used in `lib_param_bn`.
///
/// TODO: note that `lib-param-bn` currently cannot express `Dual` variant of `Monotonicity`. We convert it
/// to `Unknown` instead.
pub fn sign_to_monotonicity(regulation_sign: &Monotonicity) -> Option<Lib_Pbn_Monotonicity> {
    match regulation_sign {
        Monotonicity::Activation => Some(Lib_Pbn_Monotonicity::Activation),
        Monotonicity::Inhibition => Some(Lib_Pbn_Monotonicity::Inhibition),
        Monotonicity::Unknown => None,
        // todo: maybe put "unimplemented" here?
        Monotonicity::Dual => None,
    }
}

/// Utility method to convert `Essentiality` from boolean.
///
/// TODO: note that `lib-param-bn` currently cannot distinguish between `False` and `Unknown`
/// variants of `Essentiality`. In general, these are both represented by "false" in `lib-param-bn`.
pub fn essentiality_from_bool(essentiality: bool) -> Essentiality {
    match essentiality {
        true => Essentiality::True,
        // this is how it currently works now in `lib-param-bn`
        false => Essentiality::Unknown,
    }
}

/// Utility method to convert `Essentiality` into boolean.
///
/// TODO: note that `lib-param-bn` currently cannot distinguish between `False` and `Unknown`
/// variants of `Essentiality`. In general, these are both represented by "false" in `lib-param-bn`.
pub fn essentiality_to_bool(essentiality: Essentiality) -> bool {
    match essentiality {
        Essentiality::True => true,
        Essentiality::Unknown => false,
        Essentiality::False => false,
    }
}
