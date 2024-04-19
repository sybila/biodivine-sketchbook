use crate::sketchbook::observations::{Dataset, ObservationManager};
use crate::sketchbook::utils::{parse_map_keys, stringify_and_order_keys};

use std::collections::HashMap;
use std::fmt::{self, Formatter};

use serde::de::{self, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize, Deserializer};

/// De-serialization to convert `ObservationManager` to string.
/// Own implementation is needed as `serde` struggles with `HashMaps` with non-string keys.
impl Serialize for ObservationManager {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ObservationManager", 1)?;
        // Serialize `nodes` field (HashMap with non-String keys) as a HashMap with String keys
        let datasets = stringify_and_order_keys(&self.datasets);
        state.serialize_field("datasets", &datasets)?;

        state.end()
    }
}

/// De-serialization to construct `ObservationManager` from string.
/// Own implementation is needed as `serde` struggles with `HashMaps` with non-string keys.
impl<'de> Deserialize<'de> for ObservationManager {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Datasets,
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
                        formatter.write_str("`datasets`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "datasets" => Ok(Field::Datasets),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ObservationManagerVisitor;

        impl<'de> Visitor<'de> for ObservationManagerVisitor {
            type Value = ObservationManager;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct ObservationManager")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ObservationManager, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut datasets = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Datasets => {
                            if datasets.is_some() {
                                return Err(de::Error::duplicate_field("datasets"));
                            }
                            let d: HashMap<String, Dataset> = map.next_value()?;
                            datasets = Some(parse_map_keys(d).map_err(de::Error::custom)?);
                        }
                    }
                }

                let datasets = datasets.ok_or_else(|| de::Error::missing_field("datasets"))?;
                Ok(ObservationManager { datasets })
            }
        }

        const FIELDS: &[&str] = &["datasets"];
        deserializer.deserialize_struct("ObservationManager", FIELDS, ObservationManagerVisitor)
    }
}
