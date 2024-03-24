use crate::sketchbook::observations::Dataset;
use crate::sketchbook::utils::{parse_map_keys, stringify_map_keys};

use std::collections::HashMap;
use std::fmt::{self, Formatter};

use serde::de::{self, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize, Deserializer};

/// De-serialization to convert `Dataset` to string.
/// Own implementation is needed as `serde` struggles with `HashMaps` with non-string keys.
impl Serialize for Dataset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Dataset", 4)?;
        state.serialize_field("observations", &self.observations)?;
        state.serialize_field("variables", &self.variables)?;
        state.serialize_field("data_type", &self.data_type)?;

        // Serialize `index_map` field (HashMap with non-String keys) as a HashMap with String keys
        let index_map = stringify_map_keys(&self.index_map);
        state.serialize_field("index_map", &index_map)?;

        state.end()
    }
}

/// De-serialization to construct `Dataset` from string.
/// Own implementation is needed as `serde` struggles with `HashMaps` with non-string keys.
impl<'de> Deserialize<'de> for Dataset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Observations,
            Variables,
            DataType,
            IndexMap,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter
                            .write_str("`observations`, `variables`, `data_type` or `index_map`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "observations" => Ok(Field::Observations),
                            "variables" => Ok(Field::Variables),
                            "data_type" => Ok(Field::DataType),
                            "index_map" => Ok(Field::IndexMap),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DatasetVisitor;

        impl<'de> Visitor<'de> for DatasetVisitor {
            type Value = Dataset;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct Dataset")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Dataset, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut observations = None;
                let mut variables = None;
                let mut data_type = None;
                let mut index_map = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Observations => {
                            if observations.is_some() {
                                return Err(de::Error::duplicate_field("observations"));
                            }
                            observations = Some(map.next_value()?);
                        }
                        Field::Variables => {
                            if variables.is_some() {
                                return Err(de::Error::duplicate_field("variables"));
                            }
                            variables = Some(map.next_value()?);
                        }
                        Field::DataType => {
                            if data_type.is_some() {
                                return Err(de::Error::duplicate_field("data_type"));
                            }
                            data_type = Some(map.next_value()?);
                        }
                        Field::IndexMap => {
                            if index_map.is_some() {
                                return Err(de::Error::duplicate_field("index_map"));
                            }
                            let i_map: HashMap<String, usize> = map.next_value()?;
                            index_map = Some(parse_map_keys(i_map).map_err(de::Error::custom)?);
                        }
                    }
                }

                let observations =
                    observations.ok_or_else(|| de::Error::missing_field("observations"))?;
                let variables = variables.ok_or_else(|| de::Error::missing_field("variables"))?;
                let data_type = data_type.ok_or_else(|| de::Error::missing_field("data_type"))?;
                let index_map = index_map.ok_or_else(|| de::Error::missing_field("index_map"))?;
                Ok(Dataset {
                    observations,
                    variables,
                    data_type,
                    index_map,
                })
            }
        }

        const FIELDS: &[&str] = &["observations", "variables", "data_type", "index_map"];
        deserializer.deserialize_struct("Dataset", FIELDS, DatasetVisitor)
    }
}
