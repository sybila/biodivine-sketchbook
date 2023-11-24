use crate::sketchbook::{Monotonicity, Regulation, VarId};
use std::fmt::{Display, Error, Formatter};

use regex::Regex;

/// **(internal)** A regex string of an identifier which we currently allow to appear.
/// This regex does not enforce beginning/ending as it is used inside of larger regulation
/// regex.
const ID_REGEX_STR: &str = r"[a-zA-Z0-9_]+";

/// **(internal)** Regex which matches the regulation arrow string with `monotonicity`
/// and `observable` groups.
const REGULATION_ARROW_REGEX_STR: &str = r"-(?P<monotonicity>[|>?])(?P<observable>\??)";

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
    /// `monotonicity`, `observability` and `target`. If the string is not valid, returns `None`.
    pub fn try_from_string(
        regulation: &str,
    ) -> Result<(String, Option<Monotonicity>, bool, String), String> {
        REGULATION_REGEX
            .captures(regulation.trim())
            .map(|captures| {
                let monotonicity = match &captures["monotonicity"] {
                    "?" => None,
                    "|" => Some(Monotonicity::Inhibition),
                    ">" => Some(Monotonicity::Activation),
                    _ => unreachable!("Nothing else matches this group."),
                };
                let observable = captures["observable"].is_empty();
                (
                    captures["regulator"].to_string(),
                    monotonicity,
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

    /// Return monotonicity of the regulation (if specified).
    pub fn get_monotonicity(&self) -> Option<Monotonicity> {
        self.monotonicity
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
        let monotonicity = match self.get_monotonicity() {
            None => "?",
            Some(Monotonicity::Activation) => ">",
            Some(Monotonicity::Inhibition) => "|",
        };
        let observability = if self.is_observable() { "" } else { "?" };

        write!(
            f,
            "{} -{}{} {}",
            self.regulator, monotonicity, observability, self.target
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::{Monotonicity, Regulation, VarId};

    #[test]
    fn regulation_conversion() {
        let regulation_strings = vec![
            "a -?? b", "b -? c", "c ->? d", "d -> e", "e -|? f", "f -| g",
        ];

        let regulators = vec!["a", "b", "c", "d", "e", "f"];
        let targets = vec!["b", "c", "d", "e", "f", "g"];
        let observability = vec![false, true, false, true, false, true];
        let monotonicity = vec![
            None,
            None,
            Some(Monotonicity::Activation),
            Some(Monotonicity::Activation),
            Some(Monotonicity::Inhibition),
            Some(Monotonicity::Inhibition),
        ];

        for i in 0..regulation_strings.len() {
            let (r, m, o, t) = Regulation::try_from_string(regulation_strings[i.clone()]).unwrap();
            assert_eq!(&r, regulators[i.clone()]);
            assert_eq!(&t, targets[i.clone()]);
            assert_eq!(m, monotonicity[i.clone()]);
            assert_eq!(o, observability[i.clone()]);

            let regulation = Regulation {
                regulator: VarId::new(r.as_str()).unwrap(),
                target: VarId::new(t.as_str()).unwrap(),
                observable: o,
                monotonicity: m,
            };
            assert_eq!(regulation.to_string().as_str(), regulation_strings[i]);
        }

        assert!(Regulation::try_from_string("a --> b").is_err());
    }
}
