use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

/// Convert keys of the `HashMap` to `String`, and then order the map by converting it into
/// a sorted `BTreeMap`.
///
/// The ordering enables deterministic serialization, and the string keys are needed to
/// implement the [serde::Serialize] trait (serde requires `String` map keys).
pub fn stringify_and_order_keys<K: ToString + Ord, V>(map: &HashMap<K, V>) -> BTreeMap<String, &V> {
    map.iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect::<BTreeMap<String, &V>>()
}

/*
/// Convert the `HashMap` with ID keys into a sorted `BTreeMap` and then serialize it.
/// This enables deterministic serialization.
///
/// Use this with serde's [serialize_with] attribute.
pub fn to_ordered_map<S: Serializer, K: Ord + Serialize, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S
) -> Result<S::Ok, S::Error> {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
 */

/// Convert keys of the `HashMap` from `String` to a given type.
/// This is useful for our own implementing of the [serde::Deserialize] trait.
pub fn parse_map_keys<K, V>(map: HashMap<String, V>) -> Result<HashMap<K, V>, String>
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

/// Check if a name string is valid, return Error otherwise.
/// Currently, all names not containing newlines are valid.
pub(crate) fn assert_name_valid(name: &str) -> Result<(), String> {
    if name.contains('\n') {
        return Err("Name must not contain a newline.".to_string());
    }
    Ok(())
}

/// Check the list of (typesafe or string) IDs contains only unique IDs.
pub(crate) fn assert_ids_unique<T: Eq + Hash + Debug>(id_list: &Vec<T>) -> Result<(), String> {
    let id_set = id_list.iter().collect::<HashSet<_>>();
    if id_set.len() != id_list.len() {
        return Err(format!("List of IDs `{:?}` contain duplicates.", id_list));
    }
    Ok(())
}
