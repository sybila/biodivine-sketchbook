use crate::sketchbook::{RegulationSign, Regulation, VarId};
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

/// Serialization utility methods.
impl Regulation {
    /// Try to read all available information about a regulation from a given string
    /// in the standard format.
    ///
    /// The returned data correspond to the items as they appear in the string, i.e. `regulator`,
    /// `regulation_sign`, `observability` and `target`. If the string is not valid, returns `None`.
    pub fn try_from_string(
        regulation: &str,
    ) -> Result<(String, RegulationSign, bool, String), String> {
        REGULATION_REGEX
            .captures(regulation.trim())
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
            .ok_or(format!("Regulation string is invalid: {regulation}"))
    }
}

/// Basic getters.
impl Regulation {
    /// Check if the regulation is marked as observable.
    pub fn is_observable(&self) -> bool {
        self.observable
    }

    /// Return the sign of the regulation.
    pub fn get_sign(&self) -> RegulationSign {
        self.regulation_sign
    }

    /// Get the `VarId` of the regulator.
    pub fn get_regulator(&self) -> VarId {
        self.regulator.clone()
    }

    /// Get the `VarId` of the target.
    pub fn get_target(&self) -> VarId {
        self.target.clone()
    }
}

impl Display for Regulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let regulation_sign = match self.get_sign() {
            RegulationSign::Unknown => "?",
            RegulationSign::Activation => ">",
            RegulationSign::Inhibition => "|",
            RegulationSign::Dual => "D",
        };
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
    use crate::sketchbook::{RegulationSign, Regulation, VarId};

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
            let (r, s, o, t) = Regulation::try_from_string(regulation_strings[i.clone()]).unwrap();
            assert_eq!(&r, regulators[i.clone()]);
            assert_eq!(&t, targets[i.clone()]);
            assert_eq!(s, regulation_sign[i.clone()]);
            assert_eq!(o, observability[i.clone()]);

            let regulation = Regulation {
                regulator: VarId::new(r.as_str()).unwrap(),
                target: VarId::new(t.as_str()).unwrap(),
                observable: o,
                regulation_sign: s,
            };
            assert_eq!(regulation.to_string().as_str(), regulation_strings[i]);
        }

        assert!(Regulation::try_from_string("a --> b").is_err());
        assert!(Regulation::try_from_string("-a -> b").is_err());
        assert!(Regulation::try_from_string("a - b").is_err());
    }
}
