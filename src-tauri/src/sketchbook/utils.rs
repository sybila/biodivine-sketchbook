use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

/// Check if a name string is valid, return Error otherwise.
///
/// Currently, all names that do not contain newlines are valid.
pub(crate) fn assert_name_valid(name: &str) -> Result<(), String> {
    if name.contains('\n') {
        return Err("Name must not contain a newline.".to_string());
    }
    Ok(())
}

/// Check that the list of (typesafe or string) IDs contains only unique IDs (no duplicates).
pub(crate) fn assert_ids_unique<T: Eq + Hash + Debug>(id_list: &Vec<T>) -> Result<(), String> {
    let id_set = id_list.iter().collect::<HashSet<_>>();
    if id_set.len() != id_list.len() {
        return Err(format!("List of IDs `{:?}` contain duplicates.", id_list));
    }
    Ok(())
}

/// Convert keys of the `HashMap` to `String`, and then order the map by converting it into
/// a sorted `BTreeMap`.
///
/// The ordering enables deterministic serialization, and the string keys are needed to correctly
/// implement the [serde::Serialize] trait (`serde_json` requires `String` map keys).
pub fn _stringify_and_order_keys<K: ToString + Ord, V>(
    map: &HashMap<K, V>,
) -> BTreeMap<String, &V> {
    map.iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect::<BTreeMap<String, &V>>()
}

/// Convert keys of the `HashMap` from `String` to a given type.
/// This is useful for our own implementing of the [serde::Deserialize] trait.
pub fn _parse_map_keys<K, V>(map: HashMap<String, V>) -> Result<HashMap<K, V>, String>
where
    K: FromStr + Hash + Eq,
    <K as FromStr>::Err: std::fmt::Display,
    String: From<<K as FromStr>::Err>,
{
    let transformed_map = map
        .into_iter()
        .map(|(k, v)| k.parse::<K>().map(|k_parsed| (k_parsed, v)))
        .collect::<Result<HashMap<K, V>, _>>()?;
    Ok(transformed_map)
}
