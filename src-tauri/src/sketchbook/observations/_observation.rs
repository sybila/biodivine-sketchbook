use crate::sketchbook::observations::_var_value::VarValue;
use crate::sketchbook::ObservationId;
use biodivine_lib_param_bn::{Space, VariableId};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// A single named observation, i.e., an ordered vector of binarized values.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    id: ObservationId,
    values: Vec<VarValue>,
}

/// Creating observations.
impl Observation {
    /// Create `Observation` object from a vector with values and an ID.
    pub fn new_with_id(values: Vec<VarValue>, id: ObservationId) -> Self {
        Self { values, id }
    }

    /// Create `Observation` object from a vector with values, and string ID (which must be valid).
    pub fn new(values: Vec<VarValue>, id: &str) -> Result<Self, String> {
        Ok(Self::new_with_id(values, ObservationId::new(id)?))
    }

    /// Create `Observation` encoding a vector of `n` ones.
    pub fn new_full_ones(n: usize, id: &str) -> Result<Self, String> {
        Self::new(vec![VarValue::True; n], id)
    }

    /// Create `Observation` encoding a vector of `n` zeros.
    pub fn new_full_zeros(n: usize, id: &str) -> Result<Self, String> {
        Self::new(vec![VarValue::False; n], id)
    }

    /// Create `Observation` encoding a vector of `n` unspecified values.
    pub fn new_full_unspecified(n: usize, id: &str) -> Result<Self, String> {
        Self::new(vec![VarValue::Any; n], id)
    }

    /// Create `Observation` from a similar [Space] object of the [biodivine_lib_param_bn] library.
    /// The values of the resulting observation are given in the same order as `var_ids`.
    pub fn from_space(space: &Space, var_ids: &Vec<VariableId>, id: ObservationId) -> Observation {
        let mut vec_values = Vec::new();
        for var_id in var_ids {
            vec_values.push(VarValue::from(space[*var_id]));
        }
        Observation::new_with_id(vec_values, id)
    }

    /// Create `Observation` object from string encoding of its (ordered) values.
    /// Values are encoded using characters `1`, `0`, or `*`.
    ///
    /// Observation cannot be empty.
    pub fn try_from_str(observation_string: String, id: &str) -> Result<Self, String> {
        let mut observation_vec: Vec<VarValue> = Vec::new();
        for c in observation_string.chars() {
            observation_vec.push(VarValue::from_str(&c.to_string())?)
        }
        if observation_vec.is_empty() {
            return Err("Observation can't be empty.".to_string());
        }

        Self::new(observation_vec, id)
    }
}

/// Editing observations.
impl Observation {
    /// Set the value at given idx.
    pub fn set_value_at_idx(&mut self, index: usize, value: VarValue) -> Result<(), String> {
        if index >= self.num_values() {
            return Err("Index is larger than number of values.".to_string());
        }
        self.values[index] = value;
        Ok(())
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

    /// Set the id of this observation.
    pub fn set_id(&mut self, id: ObservationId) {
        self.id = id;
    }
}

/// Observing `Observation` instances.
impl Observation {
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
        let observation_str = "001**".to_string();
        let id = "observation_id";
        let values = vec![
            VarValue::False,
            VarValue::False,
            VarValue::True,
            VarValue::Any,
            VarValue::Any,
        ];
        let observation = Observation::new(values, id).unwrap();
        assert_eq!(
            Observation::try_from_str(observation_str, id).unwrap(),
            observation
        );
    }

    #[test]
    /// Test error handling while creating observation object from string.
    fn test_err_observation_from_str() {
        let observation_str1 = "0 1**".to_string();
        let observation_str2 = "0**a".to_string();
        let observation_str3 = "".to_string();

        assert!(Observation::try_from_str(observation_str1, "obs1").is_err());
        assert!(Observation::try_from_str(observation_str2, "obs2").is_err());
        assert!(Observation::try_from_str(observation_str3, "obs3").is_err());
    }

    #[test]
    /// Test displaying of observations.
    fn test_display_observations() {
        let values = vec![
            VarValue::False,
            VarValue::False,
            VarValue::True,
            VarValue::Any,
            VarValue::Any,
        ];
        let observation = Observation::new(values, "id1").unwrap();
        let expected_long = "id1(001**)".to_string();
        let expected_short = "001**".to_string();
        assert_eq!(observation.to_values_string(), expected_short);
        assert_eq!(observation.to_debug_string(), expected_long);
    }
}
