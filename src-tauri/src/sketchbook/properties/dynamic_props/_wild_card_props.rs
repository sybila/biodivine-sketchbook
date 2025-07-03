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
    /// The Boolean flags indicate whether to consider minimal or non-percolable trap spaces.
    TrapSpaces(DatasetId, Option<ObservationId>, bool, bool),
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

    /// Create `WildCardProposition` instance describing existence of fixed points corresponding
    /// to observations of a given dataset (or optionally its single observation).
    pub fn mk_fixed_points(
        orig_str: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
    ) -> WildCardProposition {
        let variant = WildCardType::FixedPoints(dataset, observation);
        Self::new_raw(orig_str, variant)
    }

    /// Utility to create `WildCardProposition` instance describing existence of trap spaces
    /// corresponding to a given dataset (and optionally its single observation).
    /// Trap space may be standard, minimal, or non-percolable.
    fn mk_trap_spaces(
        orig_str: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
        is_min: bool,
        is_non_perc: bool,
    ) -> WildCardProposition {
        let variant = WildCardType::TrapSpaces(dataset, observation, is_min, is_non_perc);
        Self::new_raw(orig_str, variant)
    }

    /// Create `WildCardProposition` instance describing existence of trap spaces
    /// corresponding to a given dataset (and optionally its single observation).
    ///
    /// This is a any trap space, i.e., it is not required to be minimal or not
    /// non-percolable.
    pub fn mk_general_trap_spaces(
        orig_str: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
    ) -> WildCardProposition {
        Self::mk_trap_spaces(orig_str, dataset, observation, false, false)
    }

    /// Create `WildCardProposition` instance describing existence of minimal trap spaces
    /// corresponding to observations of a given dataset (or optionally its single observation).
    pub fn mk_min_trap_spaces(
        orig_str: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
    ) -> WildCardProposition {
        // Minimal trap spaces are always also non-percolable
        Self::mk_trap_spaces(orig_str, dataset, observation, true, true)
    }

    /// Create `WildCardProposition` instance describing existence of non-percolable trap spaces
    /// corresponding to observations of a given dataset (or optionally its single observation).
    ///
    /// Non-percolable trap spaces do not need to be minimal.
    pub fn mk_non_percolable_trap_spaces(
        orig_str: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
    ) -> WildCardProposition {
        Self::mk_trap_spaces(orig_str, dataset, observation, false, true)
    }

    /// Create `WildCardProposition` instance describing existence of attractors corresponding
    /// to observations of a given dataset (or optionally its single observation).
    pub fn mk_attractors(
        orig_str: &str,
        dataset: DatasetId,
        observation: Option<ObservationId>,
    ) -> WildCardProposition {
        let variant = WildCardType::Attractors(dataset, observation);
        Self::new_raw(orig_str, variant)
    }

    /// Create `WildCardProposition` instance describing existence of a trajectory
    /// between observations of a given dataset.
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
    /// Some of the templates make it possible to omit optional arguments for convenience.
    ///
    /// Currently supported variants are:
    /// - observation in a dataset given as `datasetId, observationId`
    /// - trajectory template `trajectory(datasetId)`
    /// - attractors given as `attractors(datasetId, observationId)` or `attractors(datasetId)`
    /// - fixed points given as `fixed_points(datasetId, observationId)` or `fixed_points(datasetId)`
    /// - trap spaces given as `trap_spaces(datasetId, observationId)` or `trap_spaces(datasetId)`
    /// - minimal trap spaces given as `min_trap_spaces(datasetId, observationId)` or `min_trap_spaces(datasetId)`
    /// - non-percolable trap spaces given as `non_percolable_trap_spaces(datasetId, observationId)` or `non_percolable_trap_spaces(datasetId)`
    /// - attractor count given as `attractor_count(minimal, maximal)` or `attractor_count(number)`
    pub fn try_from_str(formula: &str) -> Result<WildCardProposition, String> {
        // regex for ID matching (valid for both dataset and observation IDs)
        let id_re: &str = r"[a-zA-Z_][a-zA-Z0-9_]*";

        // observation in a dataset given as `datasetId, observationId`
        let observation_re = Regex::new(&format!(r"^({id_re})\s*,\s*({id_re})$")).unwrap();
        // trajectory template `trajectory(datasetId)`
        let trajectory_re = Regex::new(&format!(r"^trajectory\(\s*({id_re})\s*\)$")).unwrap();
        // attractors template `attractors(datasetId, observationId)` or `attractors(datasetId)`
        let attr_re = Regex::new(&format!(
            r"^attractors\(\s*({id_re})(?:\s*,\s*({id_re}))?\s*\)$"
        ))
        .unwrap();
        // fixed points template `fixed_points(datasetId, observationId)` or `fixed_points(datasetId)`
        let fp_re = Regex::new(&format!(
            r"^fixed_points\(\s*({id_re})(?:\s*,\s*({id_re}))?\s*\)$"
        ))
        .unwrap();
        // general trap spaces template `trap_spaces(datasetId, observationId)` or `trap_spaces(datasetId)`
        let ts_re = Regex::new(&format!(
            r"^trap_spaces\(\s*({id_re})(?:\s*,\s*({id_re}))?\s*\)$"
        ))
        .unwrap();
        // minimal trap spaces template `min_trap_spaces(datasetId, observationId)` or `min_trap_spaces(datasetId)`
        let min_ts_re = Regex::new(&format!(
            r"^min_trap_spaces\(\s*({id_re})(?:\s*,\s*({id_re}))?\s*\)$"
        ))
        .unwrap();
        // non-percolable trap spaces template `non_percolable_trap_spaces(datasetId, observationId)` or `non_percolable_trap_spaces(datasetId)`
        let non_percolable_ts_re = Regex::new(&format!(
            r"^non_percolable_trap_spaces\(\s*({id_re})(?:\s*,\s*({id_re}))?\s*\)$"
        ))
        .unwrap();
        // attractor count template `attractor_count(minimal, maximal)` or `attractor_count(number)`
        let attr_count_re =
            Regex::new(r"^attractor_count\(\s*(\d+)(?:\s*,\s*(\d+))?\s*\)$").unwrap();

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
            Ok(WildCardProposition::mk_general_trap_spaces(
                formula,
                dataset_id,
                observation_id,
            ))
        } else if let Some(captures) = min_ts_re.captures(formula) {
            let dataset_id = DatasetId::new(&captures[1])?;
            let observation_id = if let Some(obs_id) = captures.get(2) {
                Some(ObservationId::new(obs_id.as_str())?)
            } else {
                None
            };
            Ok(WildCardProposition::mk_min_trap_spaces(
                formula,
                dataset_id,
                observation_id,
            ))
        } else if let Some(captures) = non_percolable_ts_re.captures(formula) {
            let dataset_id = DatasetId::new(&captures[1])?;
            let observation_id = if let Some(obs_id) = captures.get(2) {
                Some(ObservationId::new(obs_id.as_str())?)
            } else {
                None
            };
            Ok(WildCardProposition::mk_non_percolable_trap_spaces(
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
            let maximal = if let Some(max_match) = captures.get(2) {
                max_match
                    .as_str()
                    .parse::<usize>()
                    .map_err(|e| e.to_string())?
            } else {
                minimal
            };
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
            WildCardType::TrapSpaces(dat_id, obs_id, is_min, is_non_perc) => match obs_id {
                // distinguish between general/non-percolable/minimal trap spaces
                Some(obs_id) => {
                    if *is_min {
                        format!("min_trap_spaces_{dat_id}_{obs_id}")
                    } else if *is_non_perc {
                        format!("non_percolable_trap_spaces_{dat_id}_{obs_id}")
                    } else {
                        format!("trap_spaces_{dat_id}_{obs_id}")
                    }
                }
                None => {
                    if *is_min {
                        format!("min_trap_spaces_{dat_id}_all")
                    } else if *is_non_perc {
                        format!("non_percolable_trap_spaces_{dat_id}_all")
                    } else {
                        format!("trap_spaces_{dat_id}_all")
                    }
                }
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
        // normal range without spaces
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

        // normal range with spaces
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

        // only single value provided
        let prop = WildCardProposition::try_from_str("attractor_count( 7 )").unwrap();
        assert_eq!(prop.orig_string(), "attractor_count( 7 )");
        assert_eq!(prop.processed_string(), "attractor_count_7_7");
        match prop.get_prop_data() {
            WildCardType::AttractorCount(minimal, maximal) => {
                assert_eq!(*minimal, 7);
                assert_eq!(*maximal, 7);
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
            WildCardType::TrapSpaces(ds, obs, is_min, is_non_perc) => {
                assert_eq!(ds.as_str(), "d1");
                assert_eq!(obs.clone().unwrap().as_str(), "o1");
                assert!(!is_min);
                assert!(!is_non_perc);
            }
            _ => assert!(false),
        }

        // observation not specified
        let prop = WildCardProposition::try_from_str("trap_spaces(d1)").unwrap();
        assert_eq!(prop.orig_string(), "trap_spaces(d1)");
        assert_eq!(prop.processed_string(), "trap_spaces_d1_all");
        match prop.get_prop_data() {
            WildCardType::TrapSpaces(ds, obs, is_min, is_non_perc) => {
                assert_eq!(ds.as_str(), "d1");
                assert!(obs.is_none());
                assert!(!is_min);
                assert!(!is_non_perc);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_min_trap_spaces() {
        // observation specified
        let prop = WildCardProposition::try_from_str("min_trap_spaces(d1, o1)").unwrap();
        assert_eq!(prop.orig_string(), "min_trap_spaces(d1, o1)");
        assert_eq!(prop.processed_string(), "min_trap_spaces_d1_o1");
        match prop.get_prop_data() {
            WildCardType::TrapSpaces(ds, obs, is_min, is_non_perc) => {
                assert_eq!(ds.as_str(), "d1");
                assert_eq!(obs.clone().unwrap().as_str(), "o1");
                assert!(is_min);
                assert!(is_non_perc);
            }
            _ => assert!(false),
        }

        // observation not specified
        let prop = WildCardProposition::try_from_str("min_trap_spaces(d1)").unwrap();
        assert_eq!(prop.orig_string(), "min_trap_spaces(d1)");
        assert_eq!(prop.processed_string(), "min_trap_spaces_d1_all");
        match prop.get_prop_data() {
            WildCardType::TrapSpaces(ds, obs, is_min, is_non_perc) => {
                assert_eq!(ds.as_str(), "d1");
                assert!(obs.is_none());
                assert!(is_min);
                assert!(is_non_perc);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_non_percolable_trap_spaces() {
        // observation specified
        let prop = WildCardProposition::try_from_str("non_percolable_trap_spaces(d1, o1)").unwrap();
        assert_eq!(prop.orig_string(), "non_percolable_trap_spaces(d1, o1)");
        assert_eq!(prop.processed_string(), "non_percolable_trap_spaces_d1_o1");
        match prop.get_prop_data() {
            WildCardType::TrapSpaces(ds, obs, is_min, is_non_perc) => {
                assert_eq!(ds.as_str(), "d1");
                assert_eq!(obs.clone().unwrap().as_str(), "o1");
                assert!(!is_min);
                assert!(is_non_perc);
            }
            _ => assert!(false),
        }

        // observation not specified
        let prop = WildCardProposition::try_from_str("non_percolable_trap_spaces(d1)").unwrap();
        assert_eq!(prop.orig_string(), "non_percolable_trap_spaces(d1)");
        assert_eq!(prop.processed_string(), "non_percolable_trap_spaces_d1_all");
        match prop.get_prop_data() {
            WildCardType::TrapSpaces(ds, obs, is_min, is_non_perc) => {
                assert_eq!(ds.as_str(), "d1");
                assert!(obs.is_none());
                assert!(!is_min);
                assert!(is_non_perc);
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
