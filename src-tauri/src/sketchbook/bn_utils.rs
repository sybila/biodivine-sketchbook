use crate::sketchbook::model::{Essentiality, Monotonicity};
use biodivine_lib_param_bn::Monotonicity as Lib_Pbn_Monotonicity;

/// **(internal)** Static utility method to convert regulation sign given by `Monotonicity`
/// used by `lib_param_bn` into the type `Monotonicity` used here.
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

/// **(internal)** Static utility method to convert regulation sign from the type
/// `Monotonicity` used here into the type `Monotonicity` used in `lib_param_bn`.
/// TODO: note that `lib-param-bn` currently cannot express `Dual` variant of `Monotonicity` and `Unknown` is used instead.
pub fn sign_to_monotonicity(regulation_sign: &Monotonicity) -> Option<Lib_Pbn_Monotonicity> {
    match regulation_sign {
        Monotonicity::Activation => Some(Lib_Pbn_Monotonicity::Activation),
        Monotonicity::Inhibition => Some(Lib_Pbn_Monotonicity::Inhibition),
        Monotonicity::Unknown => None,
        // todo: fix
        Monotonicity::Dual => None,
    }
}

/// **(internal)** Static utility method to convert `Essentiality` from boolean.
/// TODO: note that `lib-param-bn` currently cannot distinguish between `False` and `Unknown` variants of `Essentiality`.
pub fn essentiality_from_bool(essentiality: bool) -> Essentiality {
    match essentiality {
        true => Essentiality::True,
        // todo: fix, this is how it works now in `lib-param-bn`
        false => Essentiality::Unknown,
    }
}

/// **(internal)** Static utility method to convert `Essentiality` to boolean.
/// TODO: note that `lib-param-bn` currently cannot distinguish between `False` and `Unknown` variants of `Essentiality`.
pub fn essentiality_to_bool(essentiality: Essentiality) -> bool {
    match essentiality {
        Essentiality::True => true,
        Essentiality::Unknown => false,
        Essentiality::False => false,
    }
}
