use serde::{Deserialize, Serialize};

use crate::sketchbook::ids::{DatasetId, ObservationId};
use regex::Regex;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WildCardType {
    /// An observation in a dataset.
    Observation(DatasetId, ObservationId),
    /// A trajectory given by a dataset.
    Trajectory(DatasetId),
    /// Attractors given by a dataset (or a single observation).
    Attractors(DatasetId, Option<ObservationId>),
    /// Fixed points given by a dataset (or a single observation).
    FixedPoints(DatasetId, Option<ObservationId>),
    /// Trap spaces given by a dataset (or a single observation).
    /// TODO: add other flags (minimal, non-percolable)
    TrapSpaces(DatasetId, Option<ObservationId>),
    /// Attractor count range (min, max).
    AttractorCount(usize, usize),
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
    /// **(internal)** Shorthand to create a wild-card prop given its already created
    /// internal `WildCardType` data, name, and annotation.
    fn new_raw(orig_str: &str, variant: WildCardType) -> WildCardProposition {
        WildCardProposition {
            orig_str: orig_str.to_string(),
            wild_card_type: variant,
        }
    }

    /// Create `WildCardProposition` instance describing a single observation.
    pub fn mk_observation(
        orig_str: &str,
        dataset: DatasetId,
        observation: ObservationId,
    ) -> WildCardProposition {
        let variant = WildCardType::Observation(dataset, observation);
        Self::new_raw(orig_str, variant)
    }

    /// Create `WildCardProposition` instance describing existence of fixed points
    /// corresponding to a given dataset (and optionally its single observation).
    pub fn mk_fixed_points(
        orig_str: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
    ) -> WildCardProposition {
        let variant = WildCardType::FixedPoints(dataset, observation);
        Self::new_raw(orig_str, variant)
    }

    /// Create `WildCardProposition` instance describing existence of trap spaces
    /// corresponding to a given dataset (and optionally its single observation).
    pub fn mk_trap_spaces(
        orig_str: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
    ) -> WildCardProposition {
        let variant = WildCardType::TrapSpaces(dataset, observation);
        Self::new_raw(orig_str, variant)
    }

    /// Create `WildCardProposition` instance describing existence of attractors
    /// corresponding to a given dataset (and optionally its single observation).
    pub fn mk_attractors(
        orig_str: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
    ) -> WildCardProposition {
        let variant = WildCardType::Attractors(dataset, observation);
        Self::new_raw(orig_str, variant)
    }

    /// Create `WildCardProposition` instance describing existence of a trajectory
    /// corresponding to a given dataset.
    pub fn mk_trajectory(orig_str: &str, dataset: DatasetId) -> WildCardProposition {
        let variant = WildCardType::Trajectory(dataset);
        Self::new_raw(orig_str, variant)
    }

    /// Create `WildCardProposition` instance describing attractor count.
    pub fn try_mk_attractor_count(
        orig_str: &str,
        minimal: usize,
        maximal: usize,
    ) -> Result<WildCardProposition, String> {
        if minimal > maximal {
            return Err("`minimal` attractor count cannot be larger than `maximal`.".to_string());
        }
        if minimal == 0 || maximal == 0 {
            return Err("Attractor count must be larger than 0.".to_string());
        }

        let variant = WildCardType::AttractorCount(minimal, maximal);
        Ok(Self::new_raw(orig_str, variant))
    }
}

impl WildCardProposition {
    /// Parse a single `WildCardProposition` from a string, which must be in a correct format.
    ///
    /// Currently supported variants are:
    /// - observation in a dataset given as `datasetId/observationId`
    pub fn try_from_str(formula: &str) -> Result<WildCardProposition, String> {
        let id_re: &str = r"[a-zA-Z_][a-zA-Z0-9_]*";
        let observation_re = Regex::new(&format!(r"^({id_re})\s*,\s*({id_re})$")).unwrap();
        let trajectory_re = Regex::new(&format!(r"^trajectory\(\s*({id_re})\s*\)$")).unwrap();
        let attr_re = Regex::new(&format!(
            r"^attractors\(\s*({id_re})(?:\s*,\s*({id_re}))?\s*\)$"
        ))
        .unwrap();
        let fp_re = Regex::new(&format!(
            r"^fixed_points\(\s*({id_re})(?:\s*,\s*({id_re}))?\s*\)$"
        ))
        .unwrap();
        let ts_re = Regex::new(&format!(
            r"^trap_spaces\(\s*({id_re})(?:\s*,\s*({id_re}))?\s*\)$"
        ))
        .unwrap();
        let attr_count_re = Regex::new(r"^attractor_count\(\s*(\d+)\s*,\s*(\d+)\s*\)$").unwrap();

        if let Some(captures) = observation_re.captures(formula) {
            let dataset_id = DatasetId::new(&captures[1])?;
            let observation_id = ObservationId::new(&captures[2])?;
            Ok(WildCardProposition::mk_observation(
                formula,
                dataset_id,
                observation_id,
            ))
        } else if let Some(captures) = trajectory_re.captures(formula) {
            let dataset_id = DatasetId::new(&captures[1])?;
            Ok(WildCardProposition::mk_trajectory(formula, dataset_id))
        } else if let Some(captures) = attr_re.captures(formula) {
            let dataset_id = DatasetId::new(&captures[1])?;
            let observation_id = if let Some(obs_id) = captures.get(2) {
                Some(ObservationId::new(obs_id.as_str())?)
            } else {
                None
            };
            Ok(WildCardProposition::mk_attractors(
                formula,
                dataset_id,
                observation_id,
            ))
        } else if let Some(captures) = ts_re.captures(formula) {
            let dataset_id = DatasetId::new(&captures[1])?;
            let observation_id = if let Some(obs_id) = captures.get(2) {
                Some(ObservationId::new(obs_id.as_str())?)
            } else {
                None
            };
            Ok(WildCardProposition::mk_trap_spaces(
                formula,
                dataset_id,
                observation_id,
            ))
        } else if let Some(captures) = fp_re.captures(formula) {
            let dataset_id = DatasetId::new(&captures[1])?;
            let observation_id = if let Some(obs_id) = captures.get(2) {
                Some(ObservationId::new(obs_id.as_str())?)
            } else {
                None
            };
            Ok(WildCardProposition::mk_fixed_points(
                formula,
                dataset_id,
                observation_id,
            ))
        } else if let Some(captures) = attr_count_re.captures(formula) {
            let minimal = captures[1].parse::<usize>().map_err(|e| e.to_string())?;
            let maximal = captures[2].parse::<usize>().map_err(|e| e.to_string())?;
            WildCardProposition::try_mk_attractor_count(formula, minimal, maximal)
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
            WildCardType::Trajectory(dat_id) => format!("trajectory_{dat_id}"),
            WildCardType::Attractors(dat_id, obs_id) => match obs_id {
                Some(obs_id) => format!("attractors_{dat_id}_{obs_id}"),
                None => format!("attractors_{dat_id}_all"),
            },
            WildCardType::FixedPoints(dat_id, obs_id) => match obs_id {
                Some(obs_id) => format!("fixed_points_{dat_id}_{obs_id}"),
                None => format!("fixed_points_{dat_id}_all"),
            },
            WildCardType::TrapSpaces(dat_id, obs_id) => match obs_id {
                Some(obs_id) => format!("trap_spaces_{dat_id}_{obs_id}"),
                None => format!("trap_spaces_{dat_id}_all"),
            },
            WildCardType::AttractorCount(minimal, maximal) => {
                format!("attractor_count_{minimal}_{maximal}")
            }
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
    fn test_parse_obs() {
        // valid case
        let prop = WildCardProposition::try_from_str("ds1, obs1").unwrap();
        assert_eq!(prop.orig_string(), "ds1, obs1");
        assert_eq!(prop.processed_string(), "observation_ds1_obs1");
        match prop.get_prop_data() {
            WildCardType::Observation(ds, obs) => {
                assert_eq!(ds.as_str(), "ds1");
                assert_eq!(obs.as_str(), "obs1");
            }
            _ => assert!(false),
        }

        // invalid case
        let result = WildCardProposition::try_from_str("idk_idk");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_trajectory() {
        // normal case without spaces
        let prop = WildCardProposition::try_from_str("trajectory(d1)").unwrap();
        assert_eq!(prop.orig_string(), "trajectory(d1)");
        assert_eq!(prop.processed_string(), "trajectory_d1");
        match prop.get_prop_data() {
            WildCardType::Trajectory(ds) => assert_eq!(ds.as_str(), "d1"),
            _ => assert!(false),
        }

        // normal case with spaces
        let prop = WildCardProposition::try_from_str("trajectory(   d1  )").unwrap();
        assert_eq!(prop.orig_string(), "trajectory(   d1  )");
        assert_eq!(prop.processed_string(), "trajectory_d1");
        match prop.get_prop_data() {
            WildCardType::Trajectory(ds) => assert_eq!(ds.as_str(), "d1"),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_attr_count() {
        // normal case without spaces
        let prop = WildCardProposition::try_from_str("attractor_count(2,9)").unwrap();
        assert_eq!(prop.orig_string(), "attractor_count(2,9)");
        assert_eq!(prop.processed_string(), "attractor_count_2_9");
        match prop.get_prop_data() {
            WildCardType::AttractorCount(minimal, maximal) => {
                assert_eq!(*minimal, 2);
                assert_eq!(*maximal, 9);
            }
            _ => assert!(false),
        }

        // normal case with spaces
        let prop = WildCardProposition::try_from_str("attractor_count( 8 ,  8 )").unwrap();
        assert_eq!(prop.orig_string(), "attractor_count( 8 ,  8 )");
        assert_eq!(prop.processed_string(), "attractor_count_8_8");
        match prop.get_prop_data() {
            WildCardType::AttractorCount(minimal, maximal) => {
                assert_eq!(*minimal, 8);
                assert_eq!(*maximal, 8);
            }
            _ => assert!(false),
        }

        // invalid range
        let result = WildCardProposition::try_from_str("attractor_count(9, 2)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_attractors() {
        // observation specified
        let prop = WildCardProposition::try_from_str("attractors(d1, o1)").unwrap();
        assert_eq!(prop.orig_string(), "attractors(d1, o1)");
        assert_eq!(prop.processed_string(), "attractors_d1_o1");
        match prop.get_prop_data() {
            WildCardType::Attractors(ds, obs) => {
                assert_eq!(ds.as_str(), "d1");
                assert_eq!(obs.clone().unwrap().as_str(), "o1");
            }
            _ => assert!(false),
        }

        // observation not specified
        let prop = WildCardProposition::try_from_str("attractors(d1)").unwrap();
        assert_eq!(prop.orig_string(), "attractors(d1)");
        assert_eq!(prop.processed_string(), "attractors_d1_all");
        match prop.get_prop_data() {
            WildCardType::Attractors(ds, obs) => {
                assert_eq!(ds.as_str(), "d1");
                assert!(obs.is_none());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_fixed_points() {
        // observation specified
        let prop = WildCardProposition::try_from_str("fixed_points(d1, o1)").unwrap();
        assert_eq!(prop.orig_string(), "fixed_points(d1, o1)");
        assert_eq!(prop.processed_string(), "fixed_points_d1_o1");
        match prop.get_prop_data() {
            WildCardType::FixedPoints(ds, obs) => {
                assert_eq!(ds.as_str(), "d1");
                assert_eq!(obs.clone().unwrap().as_str(), "o1");
            }
            _ => assert!(false),
        }

        // observation not specified
        let prop = WildCardProposition::try_from_str("fixed_points(d1)").unwrap();
        assert_eq!(prop.orig_string(), "fixed_points(d1)");
        assert_eq!(prop.processed_string(), "fixed_points_d1_all");
        match prop.get_prop_data() {
            WildCardType::FixedPoints(ds, obs) => {
                assert_eq!(ds.as_str(), "d1");
                assert!(obs.is_none());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_trap_spaces() {
        // observation specified
        let prop = WildCardProposition::try_from_str("trap_spaces(d1, o1)").unwrap();
        assert_eq!(prop.orig_string(), "trap_spaces(d1, o1)");
        assert_eq!(prop.processed_string(), "trap_spaces_d1_o1");
        match prop.get_prop_data() {
            WildCardType::TrapSpaces(ds, obs) => {
                assert_eq!(ds.as_str(), "d1");
                assert_eq!(obs.clone().unwrap().as_str(), "o1");
            }
            _ => assert!(false),
        }

        // observation not specified
        let prop = WildCardProposition::try_from_str("trap_spaces(d1)").unwrap();
        assert_eq!(prop.orig_string(), "trap_spaces(d1)");
        assert_eq!(prop.processed_string(), "trap_spaces_d1_all");
        match prop.get_prop_data() {
            WildCardType::TrapSpaces(ds, obs) => {
                assert_eq!(ds.as_str(), "d1");
                assert!(obs.is_none());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_wild_card_empty() {
        let result = WildCardProposition::try_from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_process_wild_cards_single() {
        let (processed, wild_cards) = process_wild_card_props("AG EF %ds1,obs1%").unwrap();
        assert_eq!(processed, "AG EF %observation_ds1_obs1%");
        assert_eq!(wild_cards.len(), 1);
        assert_eq!(wild_cards[0].orig_string(), "ds1,obs1");
        assert_eq!(wild_cards[0].processed_string(), "observation_ds1_obs1");
    }

    #[test]
    fn test_process_wild_cards_multiple() {
        let (processed, wild_cards) =
            process_wild_card_props("AG EF (%ds1,obs1% & %trajectory(ds1)%)").unwrap();
        assert_eq!(
            processed,
            "AG EF (%observation_ds1_obs1% & %trajectory_ds1%)"
        );
        assert_eq!(wild_cards.len(), 2);
        assert_eq!(wild_cards[0].orig_string(), "ds1,obs1");
        assert_eq!(wild_cards[1].orig_string(), "trajectory(ds1)");
    }

    #[test]
    fn test_process_no_wildcards() {
        let (processed, wild_cards) = process_wild_card_props("!{x}: AG EF {x}").unwrap();
        assert_eq!(processed, "!{x}: AG EF {x}");
        assert!(wild_cards.is_empty());
    }

    #[test]
    fn test_process_wild_cards_invalid() {
        // unmatched percent sign
        let result = process_wild_card_props("AG %ds1,obs1");
        assert!(result.is_err());

        // invalid string inside
        let result = process_wild_card_props("AG %ds1-obs1%");
        assert!(result.is_err());
    }
}
