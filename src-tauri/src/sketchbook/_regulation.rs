use crate::sketchbook::{RegulationSign, VarId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

use regex::Regex;

/// **(internal)** A regex string of an identifier which we currently allow to appear.
/// This regex does not enforce beginning/ending as it is used inside of larger regulation
/// regex.
const ID_REGEX_STR: &str = r"[a-zA-Z_][a-zA-Z0-9_]*";

/// **(internal)** Regex which matches the regulation arrow string with `regulation_sign`
/// and `observable` groups.
const REGULATION_ARROW_REGEX_STR: &str = r"-(?P<regulation_sign>[|>?D])(?P<observable>\??)";

lazy_static! {
    /// **(internal)** A regex which reads one line specifying a regulation.
    static ref REGULATION_REGEX: Regex = Regex::new(
        format!(
            // regulator ID, whitespace?, arrow string, whitespace?, target ID
            r"^(?P<regulator>{})\s*{}\s*(?P<target>{})$",
            ID_REGEX_STR,
            REGULATION_ARROW_REGEX_STR,
            ID_REGEX_STR,
        ).as_str()
    ).unwrap();
}

/// Describes an interaction between two variables, `regulator` and `target`.
/// Every regulation can be *monotonous* and can be set as *observable*:
///
///  - Monotonicity is either *positive* or *negative* and signifies that the influence of the
/// `regulator` on the `target` has to *increase* or *decrease* the `target` value respectively.
///  - If observability is set to `true`, the `regulator` *must* have influence on the outcome
///  of the `target` update function in *some* context. If set to false, this is not enforced
///  (i.e. the `regulator` *can* have an influence on the `target`, but it is not required).
///
/// Regulations can be represented as strings in the
/// form `"regulator_name 'relationship' target_name"`. The 'relationship' starts with `-`, which
/// is followed by `>` for activation (positive monotonicity), `|` for inhibition (negative
/// monotonicity), `D` for dual effect (non-monotonic) or `?` for unspecified monotonicity.
/// Finally, an additional `?` at the end of 'relationship' signifies a non-observable
/// (non-essential) regulation.
/// Together, this gives the following options:  `->, ->?, -|, -|?, -D, -D?, -?, -??`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Regulation {
    regulator: VarId,
    target: VarId,
    observable: bool,
    regulation_sign: RegulationSign,
}

/// Methods for safely generating new `Regulations`.
impl Regulation {
    pub fn new(
        regulator: VarId,
        target: VarId,
        observable: bool,
        regulation_sign: RegulationSign,
    ) -> Regulation {
        Regulation {
            regulator,
            target,
            observable,
            regulation_sign,
        }
    }

    /// Try to read the regulation from a given string in the standard format.
    /// Returns error if the string is invalid.
    pub fn try_from_string(regulation_str: &str) -> Result<Regulation, String> {
        let (regulator, regulation_sign, observable, target) =
            Regulation::try_components_from_string(regulation_str)?;

        Ok(Regulation {
            regulator: VarId::new(regulator.as_str())?,
            target: VarId::new(target.as_str())?,
            regulation_sign,
            observable,
        })
    }

    /// Try to read all available information about a regulation from a given string
    /// in the standard format.
    ///
    /// The returned data correspond to the items as they appear in the string, i.e. `regulator`,
    /// `regulation_sign`, `observability` and `target`. If the string is not valid, returns `None`.
    pub fn try_components_from_string(
        regulation_str: &str,
    ) -> Result<(String, RegulationSign, bool, String), String> {
        REGULATION_REGEX
            .captures(regulation_str.trim())
            .map(|captures| {
                let regulation_sign = match &captures["regulation_sign"] {
                    "|" => RegulationSign::Inhibition,
                    ">" => RegulationSign::Activation,
                    "D" => RegulationSign::Dual,
                    "?" => RegulationSign::Unknown,
                    _ => unreachable!("Nothing else matches this group."),
                };
                let observable = captures["observable"].is_empty();
                (
                    captures["regulator"].to_string(),
                    regulation_sign,
                    observable,
                    captures["target"].to_string(),
                )
            })
            .ok_or(format!("Regulation string is invalid: {regulation_str}"))
    }
}

/// Basic getters.
impl Regulation {
    /// Check if the regulation is marked as observable.
    pub fn is_observable(&self) -> bool {
        self.observable
    }

    /// Return the sign of the regulation.
    pub fn get_sign(&self) -> &RegulationSign {
        &self.regulation_sign
    }

    /// Get the `VarId` of the regulator.
    pub fn get_regulator(&self) -> &VarId {
        &self.regulator
    }

    /// Get the `VarId` of the target.
    pub fn get_target(&self) -> &VarId {
        &self.target
    }
}

/// Methods for editing `Regulations`.
impl Regulation {
    /// Directly swap original regulator with a given one.
    pub fn swap_regulator(&mut self, new_regulator: VarId) {
        self.regulator = new_regulator;
    }

    /// Directly swap original target with a given one.
    pub fn swap_target(&mut self, new_target: VarId) {
        self.target = new_target;
    }

    /// Directly swap original sign with a given one.
    pub fn swap_sign(&mut self, new_sign: RegulationSign) {
        self.regulation_sign = new_sign;
    }

    /// Directly swap original observability with a given one.
    pub fn swap_observability(&mut self, new_observability: bool) {
        self.observable = new_observability;
    }
}

impl Display for Regulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let regulation_sign = self.get_sign().to_string();
        let observability = if self.is_observable() { "" } else { "?" };

        write!(
            f,
            "{} -{}{} {}",
            self.regulator, regulation_sign, observability, self.target
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::{Regulation, RegulationSign};

    #[test]
    fn regulation_conversion() {
        let regulation_strings = vec![
            "a -?? b", "b -? c", "c ->? d", "d -> e", "e -|? f", "f -| g", "g -D? h", "h -D i",
        ];

        let regulators = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        let targets = vec!["b", "c", "d", "e", "f", "g", "h", "i"];
        let observability = vec![false, true, false, true, false, true, false, true];
        let regulation_sign = vec![
            RegulationSign::Unknown,
            RegulationSign::Unknown,
            RegulationSign::Activation,
            RegulationSign::Activation,
            RegulationSign::Inhibition,
            RegulationSign::Inhibition,
            RegulationSign::Dual,
            RegulationSign::Dual,
        ];

        for i in 0..regulation_strings.len() {
            let regulation = Regulation::try_from_string(regulation_strings[i.clone()]).unwrap();
            assert_eq!(regulation.to_string().as_str(), regulation_strings[i]);

            assert_eq!(regulation.regulator.as_str(), regulators[i.clone()]);
            assert_eq!(regulation.target.as_str(), targets[i.clone()]);
            assert_eq!(regulation.regulation_sign, regulation_sign[i.clone()]);
            assert_eq!(regulation.observable, observability[i.clone()]);
        }

        assert!(Regulation::try_from_string("a --> b").is_err());
        assert!(Regulation::try_from_string("-a -> b").is_err());
        assert!(Regulation::try_from_string("a - b").is_err());
    }
}
