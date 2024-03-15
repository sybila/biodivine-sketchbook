use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;

/// Convert keys of the `HashMap` to `String`.
/// This is useful for our own implementing of the [serde::Serialize] trait.
pub fn stringify_map_keys<K, V>(map: &HashMap<K, V>) -> HashMap<String, &V>
where
    K: ToString,
{
    map.iter().map(|(k, v)| (k.to_string(), v)).collect()
}

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
