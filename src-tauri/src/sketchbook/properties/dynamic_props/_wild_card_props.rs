use serde::{Deserialize, Serialize};

use crate::sketchbook::ids::{DatasetId, ObservationId};
use regex::Regex;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WildCardType {
    /// Wild-card proposition to represent an observation in a dataset.
    Observation(DatasetId, ObservationId),
}

/// A typesafe representation of a HCTL formula used in dynamic properties.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct WildCardProposition {
    /// Raw string parsed from the formula. This is shown on the UI side.
    orig_str: String,
    /// Processed variant of the string to pass to the model checker.
    wild_card_type: WildCardType,
}

impl WildCardProposition {
    /// Parse a single `WildCardProposition` from a string, which must be in a correct format.
    ///
    /// Currently supported variants are:
    /// - observation in a dataset given as `datasetId/observationId`
    pub fn try_from_str(formula: &str) -> Result<WildCardProposition, String> {
        let id_re: &str = r"[a-zA-Z_][a-zA-Z0-9_]*";
        let obs_re = Regex::new(&format!(r"^({id_re})/({id_re})$")).unwrap();

        if let Some(captures) = obs_re.captures(formula) {
            let dataset_id = DatasetId::new(&captures[1])?;
            let observation_id = ObservationId::new(&captures[2])?;
            Ok(WildCardProposition {
                orig_str: formula.to_string(),
                wild_card_type: WildCardType::Observation(dataset_id, observation_id),
            })
        } else {
            Err(format!(
                "Invalid wild-card proposition format - `{formula}`"
            ))
        }
    }

    pub fn orig_string(&self) -> String {
        self.orig_str.clone()
    }

    /// Get a processed variant of this property (may differ from the original string).
    pub fn processed_string(&self) -> String {
        match &self.wild_card_type {
            WildCardType::Observation(dat_id, obs_id) => format!("observation_{dat_id}_{obs_id}"),
        }
    }

    /// Get the variant with all the underlying data.
    pub fn get_prop_data(&self) -> &WildCardType {
        &self.wild_card_type
    }
}

/// Collect all wild-card proposition strings from the formula (all strings enclosed
/// in % chars) and replace them with their processed version. Return the modified
/// formula and all the collected wild cards (without the %, just the inner strings).
pub fn process_wild_card_props(
    formula: &str,
) -> Result<(String, Vec<WildCardProposition>), String> {
    let mut wild_cards = Vec::new();
    let mut result = String::new();
    let mut rest = formula;

    while let Some(start) = rest.find('%') {
        // Find the closing '%' after the opening one
        if let Some(end) = rest[start + 1..].find('%') {
            let end = start + 1 + end;
            // Push text before the wild card
            result.push_str(&rest[..start]);
            // Extract and parse the wild card proposition
            let prop_str = &rest[start + 1..end];
            let wild_card = WildCardProposition::try_from_str(prop_str)?;
            result.push_str(&format!("%{}%", wild_card.processed_string()));
            wild_cards.push(wild_card);
            // Move to the rest of the string after the closing '%'
            rest = &rest[end + 1..];
        } else {
            return Err("Unmatched '%' in the formula".to_string());
        }
    }
    // Push any remaining text after the last wild card
    result.push_str(rest);

    Ok((result, wild_cards))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wild_card() {
        let prop = WildCardProposition::try_from_str("ds1/obs1").unwrap();
        assert_eq!(prop.orig_string(), "ds1/obs1");
        assert_eq!(prop.processed_string(), "observation_ds1_obs1");
        match prop.get_prop_data() {
            WildCardType::Observation(ds, obs) => {
                assert_eq!(ds.as_str(), "ds1");
                assert_eq!(obs.as_str(), "obs1");
            }
        }
    }

    #[test]
    fn test_parse_wild_card_invalid() {
        let result = WildCardProposition::try_from_str("ds1-obs1");
        assert!(result.is_err());

        let result = WildCardProposition::try_from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_process_wild_cards_single() {
        let (processed, wild_cards) = process_wild_card_props("AG EF %ds1/obs1%").unwrap();
        assert_eq!(processed, "AG EF %observation_ds1_obs1%");
        assert_eq!(wild_cards.len(), 1);
        assert_eq!(wild_cards[0].orig_string(), "ds1/obs1");
        assert_eq!(wild_cards[0].processed_string(), "observation_ds1_obs1");
    }

    #[test]
    fn test_process_wild_cards_multiple() {
        let (processed, wild_cards) =
            process_wild_card_props("AG EF (%ds1/obs1% & %ds2/obs2%)").unwrap();
        assert_eq!(
            processed,
            "AG EF (%observation_ds1_obs1% & %observation_ds2_obs2%)"
        );
        assert_eq!(wild_cards.len(), 2);
        assert_eq!(wild_cards[0].orig_string(), "ds1/obs1");
        assert_eq!(wild_cards[1].orig_string(), "ds2/obs2");
    }

    #[test]
    fn test_process_wild_cards_no_wildcards() {
        let (processed, wild_cards) = process_wild_card_props("!{x}: AG EF {x}").unwrap();
        assert_eq!(processed, "!{x}: AG EF {x}");
        assert!(wild_cards.is_empty());
    }

    #[test]
    fn test_process_wild_cards_invalid() {
        // unmatched percent sign
        let result = process_wild_card_props("AG %ds1/obs1");
        assert!(result.is_err());

        // invalid string inside
        let result = process_wild_card_props("AG %ds1-obs1%");
        assert!(result.is_err());
    }
}
