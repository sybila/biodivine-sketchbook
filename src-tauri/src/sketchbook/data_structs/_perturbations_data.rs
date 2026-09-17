use crate::sketchbook::ids::{PerturbationId, VarId};
use crate::sketchbook::perturbations::Perturbation;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

/// Structure for sending data about `Perturbation` to the frontend.
///
/// `PerturbationData` contains similar fields as `Perturbation` but with simplified types
/// for easier serialization. The `perturbed_vars` is represented as a map of variable IDs
/// (as strings) to their perturbation values (boolean).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerturbationData {
    pub id: String,
    pub name: String,
    pub annotation: String,
    #[serde(serialize_with = "map_to_pairs", deserialize_with = "pairs_to_map")]
    pub perturbed_vars: BTreeMap<String, bool>,
}

impl JsonSerde<'_> for PerturbationData {}

impl PerturbationData {
    /// Create new `PerturbationData` object given an id, name, annotation, and perturbed variables.
    pub fn new(
        id: &str,
        name: &str,
        annotation: &str,
        perturbed_vars: BTreeMap<String, bool>,
    ) -> PerturbationData {
        PerturbationData {
            id: id.to_string(),
            name: name.to_string(),
            annotation: annotation.to_string(),
            perturbed_vars,
        }
    }

    /// Create new `PerturbationData` from a reference to a `Perturbation` and its `PerturbationId`.
    pub fn from_perturbation(
        pert_id: &PerturbationId,
        perturbation: &Perturbation,
    ) -> PerturbationData {
        let perturbed_vars = perturbation
            .get_perturbed_vars()
            .iter()
            .map(|(var_id, value)| (var_id.to_string(), *value))
            .collect();

        PerturbationData {
            id: pert_id.as_str().to_string(),
            name: perturbation.get_name().to_string(),
            annotation: perturbation.get_annotation().to_string(),
            perturbed_vars,
        }
    }

    /// Extract new `Perturbation` instance from this data.
    ///
    /// Converts string-based variable IDs to `VarId` objects.
    pub fn to_perturbation(&self) -> Result<Perturbation, String> {
        let mut perturbed_vars: BTreeMap<VarId, bool> = BTreeMap::new();

        for (var_id_str, value) in &self.perturbed_vars {
            let var_id = VarId::new(var_id_str)?;
            perturbed_vars.insert(var_id, *value);
        }

        Ok(Perturbation::new(&self.name, perturbed_vars).with_annotation(&self.annotation))
    }
}

// Helper function to serialize map as a sequence of pairs
fn map_to_pairs<K, V, S>(map: &BTreeMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize,
    V: Serialize,
    S: Serializer,
{
    serializer.collect_seq(map.iter())
}

// Helper function to deserialize sequence of pairs as a map
fn pairs_to_map<'de, K, V, D>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
where
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let pairs: Vec<(K, V)> = Vec::deserialize(deserializer)?;
    Ok(pairs.into_iter().collect())
}
