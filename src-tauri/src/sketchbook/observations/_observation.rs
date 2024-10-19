use crate::sketchbook::observations::_var_value::VarValue;
use crate::sketchbook::{ids::ObservationId, utils::assert_name_valid};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// A single named observation, i.e., an ordered vector of binarized values.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    id: ObservationId,
    name: String,
    annotation: String,
    values: Vec<VarValue>,
}

/// Creating observations.
impl Observation {
    /// Create `Observation` object from a vector with values, and string ID (which must be
    /// a valid identifier).
    ///
    /// Name is initialized same as ID, and annotation is empty.
    /// For full initializer with name and annotation, check [Self::new_annotated].
    pub fn new(values: Vec<VarValue>, id: &str) -> Result<Self, String> {
        Self::new_annotated(values, id, id, "")
    }

    /// Create `Observation` object from a vector with values, string ID (which must be
    /// a valid identifier), string name, and annotation.
    pub fn new_annotated(
        values: Vec<VarValue>,
        id: &str,
        name: &str,
        annot: &str,
    ) -> Result<Self, String> {
        assert_name_valid(name)?;
        Ok(Self {
            id: ObservationId::new(id)?,
            name: name.to_string(),
            annotation: annot.to_string(),
            values,
        })
    }

    /// Create `Observation` encoding a vector of `n` ones.
    /// Name is initialized same as ID, and annotation is empty.
    pub fn new_full_ones(n: usize, id: &str) -> Result<Self, String> {
        Self::new(vec![VarValue::True; n], id)
    }

    /// Create `Observation` encoding a vector of `n` zeros.
    /// Name is initialized same as ID, and annotation is empty.
    pub fn new_full_zeros(n: usize, id: &str) -> Result<Self, String> {
        Self::new(vec![VarValue::False; n], id)
    }

    /// Create `Observation` encoding a vector of `n` unspecified values.
    /// Name is initialized same as ID, and annotation is empty.
    pub fn new_full_unspecified(n: usize, id: &str) -> Result<Self, String> {
        Self::new(vec![VarValue::Any; n], id)
    }

    /// Create `Observation` object from string encoding of its (ordered) values.
    /// Values are encoded using characters `1`, `0`, or `*`.
    ///
    /// Observation cannot be empty. Name is initialized same as ID, and annotation is empty.
    /// For full initializer with name and annotation, check [Self::try_from_str_annotated].
    pub fn try_from_str(observation_str: &str, id: &str) -> Result<Self, String> {
        let mut observation_vec: Vec<VarValue> = Vec::new();
        for c in observation_str.chars() {
            observation_vec.push(VarValue::from_str(&c.to_string())?)
        }
        if observation_vec.is_empty() {
            return Err("Observation can't be empty.".to_string());
        }

        Self::new(observation_vec, id)
    }

    /// Create `Observation` object from string encoding of its (ordered) values.
    /// Values are encoded using characters `1`, `0`, or `*`.
    ///
    /// Observation cannot be empty.
    pub fn try_from_str_annotated(
        observation_str: &str,
        id: &str,
        name: &str,
        annot: &str,
    ) -> Result<Self, String> {
        let mut obs = Self::try_from_str(observation_str, id)?;
        obs.set_name(name)?;
        obs.set_annotation(annot);
        Ok(obs)
    }
}

/// Editing observations.
impl Observation {
    /// Set name.
    pub fn set_name(&mut self, name: &str) -> Result<(), String> {
        assert_name_valid(name)?;
        self.name = name.to_string();
        Ok(())
    }

    /// Set annotation string.
    pub fn set_annotation(&mut self, annotation: &str) {
        self.annotation = annotation.to_string();
    }

    /// Set the value at given idx.
    pub fn set_value(&mut self, index: usize, value: VarValue) -> Result<(), String> {
        if index >= self.num_values() {
            return Err("Index is larger than number of values.".to_string());
        }
        self.values[index] = value;
        Ok(())
    }

    /// Set the value (one of the "0"/"1"/"*") at given idx.
    pub fn set_value_by_str(&mut self, index: usize, value: &str) -> Result<(), String> {
        let converted_value = VarValue::from_str(value)?;
        self.set_value(index, converted_value)
    }

    /// Set all the values in this observation. The new vector of values must have the same
    /// number of values as the original observation ("arity" does not change).
    pub fn set_all_values(&mut self, values: Vec<VarValue>) -> Result<(), String> {
        if values.len() != self.num_values() {
            return Err("Vectors of old and new values differ in length.".to_string());
        }
        self.values = values;
        Ok(())
    }

    /// Set all the values in this observation via its string encoding (string of "0"/"1"/"*").
    /// The new vector of values must have the same number of values as the original observation
    /// ("arity" does not change).
    pub fn set_all_values_by_str(&mut self, values: &str) -> Result<(), String> {
        let mut converted_values: Vec<VarValue> = Vec::new();
        for c in values.chars() {
            converted_values.push(VarValue::from_str(&c.to_string())?)
        }
        self.set_all_values(converted_values)
    }

    /// Set the id of this observation.
    pub fn set_id(&mut self, id: ObservationId) {
        self.id = id;
    }

    /// Set the id of this observation, given the potential ID as string.
    pub fn set_id_by_str(&mut self, id: &str) -> Result<(), String> {
        let obs_id = ObservationId::new(id)?;
        self.set_id(obs_id);
        Ok(())
    }

    /// Remove value on given `index`, decrementing the dimension of the observation.
    pub fn remove_nth_value(&mut self, index: usize) -> Result<(), String> {
        if index >= self.num_values() {
            return Err("Index is larger than number of values.".to_string());
        }
        self.values.remove(index);
        Ok(())
    }

    /// Insert value on given `index`, incrementing the dimension of the observation.
    pub fn add_value(&mut self, index: usize, value: VarValue) -> Result<(), String> {
        if index > self.num_values() {
            return Err("Index is larger than number of values.".to_string());
        }
        self.values.insert(index, value);
        Ok(())
    }
}

/// Observing `Observation` instances.
impl Observation {
    /// Get observation's name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Annotation string of the observation.
    pub fn get_annotation(&self) -> &str {
        &self.annotation
    }

    /// Get reference to observation's vector of values.
    pub fn get_values(&self) -> &Vec<VarValue> {
        &self.values
    }

    /// Get reference to observation's id.
    pub fn get_id(&self) -> &ObservationId {
        &self.id
    }

    /// Number of all values in this observation (its "length").
    pub fn num_values(&self) -> usize {
        self.values.len()
    }

    /// Number of unspecified values in this observation.
    pub fn num_unspecified_values(&self) -> usize {
        self.values.iter().filter(|&v| *v == VarValue::Any).count()
    }

    /// Number of specified values in this observation.
    pub fn num_specified_values(&self) -> usize {
        self.values.iter().filter(|&v| *v != VarValue::Any).count()
    }

    /// Number of ones (`true` values) in this observation.
    pub fn num_ones(&self) -> usize {
        self.values.iter().filter(|&v| *v == VarValue::True).count()
    }

    /// Number of zeros (`false` values) in this observation.
    pub fn num_zeros(&self) -> usize {
        self.values
            .iter()
            .filter(|&v| *v == VarValue::False)
            .count()
    }

    /// Value at given index.
    pub fn value_at_idx(&self, index: usize) -> Result<&VarValue, String> {
        if index >= self.num_values() {
            return Err("Index is larger than number of values.".to_string());
        }
        Ok(&self.values[index])
    }

    /// Make a string with bit-encoding of values of this `Observation`.
    /// Values are encoded using characters `1`, `0`, or `*`.
    pub fn to_values_string(&self) -> String {
        let mut values_string = String::new();
        self.values
            .iter()
            .for_each(|v| values_string.push_str(v.as_str()));
        values_string
    }

    /// Make a string describing this `Observation` in a human-readable format.
    /// The format consists of id and values - `id(values)`.
    ///
    /// This is mainly for debug purposes, as it is different than classical string serialization.
    pub fn to_debug_string(&self) -> String {
        let values_string = self.to_values_string();
        format!("{}({values_string})", self.id)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::observations::{Observation, VarValue};

    #[test]
    /// Test creating observation object from string.
    fn test_observation_from_str() {
        let observation_str = "001**";
        let id = "observation_id";
        let expected_values = vec![
            VarValue::False,
            VarValue::False,
            VarValue::True,
            VarValue::Any,
            VarValue::Any,
        ];
        let expected_obs = Observation::new(expected_values, id).unwrap();
        assert_eq!(
            Observation::try_from_str(observation_str, id).unwrap(),
            expected_obs
        );
    }

    #[test]
    /// Test creating observations via provided shortcuts.
    fn test_creating_shortcuts() {
        let obs = Observation::new_full_ones(4, "o").unwrap();
        let expected_obs = Observation::try_from_str("1111", "o").unwrap();
        assert_eq!(obs, expected_obs);

        let obs = Observation::new_full_zeros(4, "o").unwrap();
        let expected_obs = Observation::try_from_str("0000", "o").unwrap();
        assert_eq!(obs, expected_obs);

        let obs = Observation::new_full_unspecified(4, "o").unwrap();
        let expected_obs = Observation::try_from_str("****", "o").unwrap();
        assert_eq!(obs, expected_obs);
    }

    #[test]
    /// Test getters and similar methods.
    fn test_getters() {
        let obs = Observation::try_from_str("10*11*", "o").unwrap();
        assert_eq!(obs.num_values(), 6);
        assert_eq!(obs.num_ones(), 3);
        assert_eq!(obs.num_zeros(), 1);
        assert_eq!(obs.num_specified_values(), 4);
        assert_eq!(obs.num_unspecified_values(), 2);

        assert_eq!(obs.get_id().as_str(), "o");
        assert_eq!(obs.value_at_idx(0).unwrap().as_str(), "1");
        assert_eq!(obs.value_at_idx(5).unwrap().as_str(), "*");
        assert!(obs.value_at_idx(6).is_err());
    }

    #[test]
    /// Test setters (for ID and values).
    fn test_setters() {
        let mut obs = Observation::try_from_str("10*11*", "o").unwrap();
        obs.set_id_by_str("p").unwrap();
        assert_eq!(obs.get_id().as_str(), "p");

        obs.set_value_by_str(1, "1").unwrap();
        assert_eq!(obs.to_values_string().as_str(), "11*11*");

        obs.set_all_values_by_str("111111").unwrap();
        assert_eq!(obs.to_values_string().as_str(), "111111");
    }

    #[test]
    /// Test inserting and removing values.
    fn test_insert_remove_value() {
        let mut observation = Observation::try_from_str("001**", "id1").unwrap();
        observation.add_value(1, VarValue::True).unwrap();
        assert_eq!(observation.to_values_string(), "0101**");

        observation.remove_nth_value(3).unwrap();
        assert_eq!(observation.to_values_string(), "010**");
    }

    #[test]
    /// Test error handling while creating observation object from string.
    fn test_err_observation_from_str() {
        let observation_str1 = "0 1**";
        let observation_str2 = "0**a";
        let observation_str3 = "";

        assert!(Observation::try_from_str(observation_str1, "obs1").is_err());
        assert!(Observation::try_from_str(observation_str2, "obs2").is_err());
        assert!(Observation::try_from_str(observation_str3, "obs3").is_err());
    }

    #[test]
    /// Test displaying of observations.
    fn test_display_observations() {
        let values_str = "001**";
        let observation = Observation::try_from_str(values_str, "id1").unwrap();
        let expected_long = "id1(001**)".to_string();
        assert_eq!(observation.to_values_string(), values_str.to_string());
        assert_eq!(observation.to_debug_string(), expected_long);
    }
}
