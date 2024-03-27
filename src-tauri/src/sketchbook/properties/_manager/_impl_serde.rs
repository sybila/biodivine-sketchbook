use crate::sketchbook::properties::{DynamicProperty, PropertyManager};
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
        let properties = stringify_and_order_keys(&self.properties);
        state.serialize_field("properties", &properties)?;

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
            Properties,
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
                        formatter.write_str("`properties`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "properties" => Ok(Field::Properties),
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
                let mut properties = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Properties => {
                            if properties.is_some() {
                                return Err(de::Error::duplicate_field("properties"));
                            }
                            let d: HashMap<String, DynamicProperty> = map.next_value()?;
                            properties = Some(parse_map_keys(d).map_err(de::Error::custom)?);
                        }
                    }
                }

                let properties =
                    properties.ok_or_else(|| de::Error::missing_field("properties"))?;
                Ok(PropertyManager { properties })
            }
        }

        const FIELDS: &[&str] = &["properties"];
        deserializer.deserialize_struct("PropertyManager", FIELDS, PropertyManagerVisitor)
    }
}
