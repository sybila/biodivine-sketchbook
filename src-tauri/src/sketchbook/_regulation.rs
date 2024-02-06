use crate::sketchbook::{Essentiality, RegulationSign, VarId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

use regex::Regex;

/// **(internal)** A regex string of an identifier which we currently allow to appear.
/// This regex does not enforce beginning/ending as it is used inside of larger regulation
/// regex.
const ID_REGEX_STR: &str = r"[a-zA-Z_][a-zA-Z0-9_]*";

/// **(internal)** Regex which matches the regulation arrow string with `regulation_sign`
/// and `essential` groups.
const REGULATION_ARROW_REGEX_STR: &str = r"-(?P<regulation_sign>[|>?*])(?P<essential>X|\?|)";

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
/// Every regulation can be *monotonous* and can be set as *essential*:
///
/// Monotonicity is `positive`, `negative`, `dual`, or `unknown`. The monotonicity signifies how
/// the presence of the `regulator` affects the value of the `target`:
///  - if the regulation is `positive`, it might only *increase* the `target` value
///  - if the regulation is `negative`, it might only *decrease* the `target` value
///  - if the regulation is `dual`, it might both *increase* or *decrease* the `target` value (in
///  different contexts)
///
/// If essentiality is set to *true*, the `regulator` *must* have influence on the outcome
/// of the `target` update function in *some* context. If set to `False`, this regulation must have
/// no effect. If it is `Unknown`, the essentiality is not enforced (i.e. the `regulator` *can*
/// have an influence on the `target`, but it is not required).
///
/// Regulations can be represented as strings in the
/// form `"regulator_name 'relationship' target_name"`. The 'relationship' starts with `-`, which
/// is followed by `>` for activation (positive monotonicity), `|` for inhibition (negative
/// monotonicity), `*` for dual effect (non-monotonic) or `?` for unspecified monotonicity.
/// Finally, an additional `X`, `?` at the end of 'relationship' signifies that the the regulation
/// is non-essential (non-essential) or the essentiality is unknown, respectively.
/// Together, this gives the following options:  `->, ->?, -|, -|?, -*, -*?, -?, -??`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Regulation {
    regulator: VarId,
    target: VarId,
    essential: Essentiality,
    regulation_sign: RegulationSign,
}

/// Methods for safely generating new `Regulations`.
impl Regulation {
    /// Create new `Regulation` given all the components.
    pub fn new(
        regulator: VarId,
        target: VarId,
        essential: Essentiality,
        regulation_sign: RegulationSign,
    ) -> Regulation {
        Regulation {
            regulator,
            target,
            essential,
            regulation_sign,
        }
    }

    /// Try to read the regulation from a given string in the standard format.
    /// Returns error if the string is invalid.
    pub fn try_from_string(regulation_str: &str) -> Result<Regulation, String> {
        let (regulator, regulation_sign, essential, target) =
            Regulation::try_components_from_string(regulation_str)?;

        Ok(Regulation {
            regulator: VarId::new(regulator.as_str())?,
            target: VarId::new(target.as_str())?,
            regulation_sign,
            essential,
        })
    }

    /// Try to read all available information about a regulation from a given string
    /// in the standard format.
    ///
    /// The returned data correspond to the items as they appear in the string, i.e. `regulator`,
    /// `regulation_sign`, `essentiality` and `target`. If the string is not valid, returns `None`.
    pub fn try_components_from_string(
        regulation_str: &str,
    ) -> Result<(String, RegulationSign, Essentiality, String), String> {
        REGULATION_REGEX
            .captures(regulation_str.trim())
            .map(|captures| {
                let regulation_sign = match &captures["regulation_sign"] {
                    "|" => RegulationSign::Inhibition,
                    ">" => RegulationSign::Activation,
                    "*" => RegulationSign::Dual,
                    "?" => RegulationSign::Unknown,
                    _ => unreachable!("Nothing else matches this group."),
                };
                let essential = match &captures["essential"] {
                    "" => Essentiality::True,
                    "X" => Essentiality::False,
                    "?" => Essentiality::Unknown,
                    _ => unreachable!("Nothing else matches this group."),
                };
                (
                    captures["regulator"].to_string(),
                    regulation_sign,
                    essential,
                    captures["target"].to_string(),
                )
            })
            .ok_or(format!("Regulation string is invalid: {regulation_str}"))
    }
}

/// Basic getters and other non-modifying methods.
impl Regulation {
    /// Check if the regulation is marked as essential.
    ///
    /// Note that both negative or unknown essentiality results in `false`.
    pub fn is_essential(&self) -> bool {
        self.essential == Essentiality::True
    }

    /// Get the essentiality of the regulation.
    pub fn get_essentiality(&self) -> &Essentiality {
        &self.essential
    }

    /// Get the sign of the regulation.
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

    /// Directly swap original essentiality with a given one.
    pub fn swap_essentiality(&mut self, new_essentiality: Essentiality) {
        self.essential = new_essentiality;
    }
}

impl Display for Regulation {
    /// Standard format that can be parsed back.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let regulation_sign = self.get_sign().to_string();
        let essentiality = match self.get_essentiality() {
            Essentiality::True => "",
            Essentiality::False => "X",
            Essentiality::Unknown => "?",
        };

        write!(
            f,
            "{} -{}{} {}",
            self.regulator, regulation_sign, essentiality, self.target
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::{Essentiality, Regulation, RegulationSign};

    #[test]
    fn regulation_conversion() {
        let regulation_strings = vec![
            "a -?? b", "b -? c", "c ->? d", "d -> e", "e -|? f", "f -| g", "g -*? h", "h -* i",
        ];

        let regulators = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        let targets = vec!["b", "c", "d", "e", "f", "g", "h", "i"];
        let essentiality = vec![
            Essentiality::Unknown,
            Essentiality::True,
            Essentiality::Unknown,
            Essentiality::True,
            Essentiality::Unknown,
            Essentiality::True,
            Essentiality::Unknown,
            Essentiality::True,
        ];
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
            assert_eq!(regulation.essential, essentiality[i.clone()]);
        }

        assert!(Regulation::try_from_string("a --> b").is_err());
        assert!(Regulation::try_from_string("-a -> b").is_err());
        assert!(Regulation::try_from_string("a - b").is_err());
    }
}
