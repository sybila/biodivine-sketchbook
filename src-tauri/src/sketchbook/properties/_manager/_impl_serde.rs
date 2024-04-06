use crate::sketchbook::properties::{DynProperty, PropertyManager, StatProperty};
use crate::sketchbook::utils::{parse_map_keys, stringify_and_order_keys};

use std::collections::HashMap;
use std::fmt::{self, Formatter};

use serde::de::{self, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize, Deserializer};

/// De-serialization to convert `PropertyManager` to string.
/// Own implementation is needed as `serde` struggles with `HashMaps` with non-string keys.
impl Serialize for PropertyManager {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PropertyManager", 1)?;
        // Serialize `nodes` field (HashMap with non-String keys) as a HashMap with String keys
        let dyn_properties = stringify_and_order_keys(&self.dyn_properties);
        state.serialize_field("dyn_properties", &dyn_properties)?;
        let stat_properties = stringify_and_order_keys(&self.stat_properties);
        state.serialize_field("stat_properties", &stat_properties)?;

        state.end()
    }
}

/// De-serialization to construct `PropertyManager` from string.
/// Own implementation is needed as `serde` struggles with `HashMaps` with non-string keys.
impl<'de> Deserialize<'de> for PropertyManager {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            DynProperties,
            StatProperties,
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
                        formatter.write_str("`dyn_properties` or `stat_properties`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "dyn_properties" => Ok(Field::DynProperties),
                            "stat_properties" => Ok(Field::StatProperties),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct PropertyManagerVisitor;

        impl<'de> Visitor<'de> for PropertyManagerVisitor {
            type Value = PropertyManager;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct PropertyManager")
            }

            fn visit_map<V>(self, mut map: V) -> Result<PropertyManager, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut dyn_properties = None;
                let mut stat_properties = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::DynProperties => {
                            if dyn_properties.is_some() {
                                return Err(de::Error::duplicate_field("dyn_properties"));
                            }
                            let d: HashMap<String, DynProperty> = map.next_value()?;
                            dyn_properties = Some(parse_map_keys(d).map_err(de::Error::custom)?);
                        }
                        Field::StatProperties => {
                            if stat_properties.is_some() {
                                return Err(de::Error::duplicate_field("stat_properties"));
                            }
                            let s: HashMap<String, StatProperty> = map.next_value()?;
                            stat_properties = Some(parse_map_keys(s).map_err(de::Error::custom)?);
                        }
                    }
                }

                let dyn_properties =
                    dyn_properties.ok_or_else(|| de::Error::missing_field("dyn_properties"))?;
                let stat_properties =
                    stat_properties.ok_or_else(|| de::Error::missing_field("stat_properties"))?;
                Ok(PropertyManager {
                    dyn_properties,
                    stat_properties,
                })
            }
        }

        const FIELDS: &[&str] = &["dyn_properties", "stat_properties"];
        deserializer.deserialize_struct("PropertyManager", FIELDS, PropertyManagerVisitor)
    }
}
