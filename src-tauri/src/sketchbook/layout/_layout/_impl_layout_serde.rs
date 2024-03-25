use crate::sketchbook::layout::{Layout, LayoutNode};
use crate::sketchbook::utils::{parse_map_keys, stringify_and_order_keys};

use std::collections::HashMap;
use std::fmt::{self, Formatter};

use serde::de::{self, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize, Deserializer};

/// De-serialization to convert `Layout` to string.
/// Own implementation is needed as `serde` struggles with `HashMaps` with non-string keys.
impl Serialize for Layout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Layout", 2)?;
        state.serialize_field("name", &self.name)?;

        // Serialize `nodes` field (HashMap with non-String keys) as a HashMap with String keys
        let nodes_map = stringify_and_order_keys(&self.nodes);
        state.serialize_field("nodes", &nodes_map)?;

        state.end()
    }
}

/// De-serialization to construct `Layout` from string.
/// Own implementation is needed as `serde` struggles with `HashMaps` with non-string keys.
impl<'de> Deserialize<'de> for Layout {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Name,
            Nodes,
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
                        formatter.write_str("`name` or `nodes`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "name" => Ok(Field::Name),
                            "nodes" => Ok(Field::Nodes),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct LayoutVisitor;

        impl<'de> Visitor<'de> for LayoutVisitor {
            type Value = Layout;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct Layout")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Layout, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut nodes = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Nodes => {
                            if nodes.is_some() {
                                return Err(de::Error::duplicate_field("nodes"));
                            }
                            let n: HashMap<String, LayoutNode> = map.next_value()?;
                            nodes = Some(parse_map_keys(n).map_err(de::Error::custom)?);
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let nodes = nodes.ok_or_else(|| de::Error::missing_field("nodes"))?;
                Ok(Layout { name, nodes })
            }
        }

        const FIELDS: &[&str] = &["name", "nodes"];
        deserializer.deserialize_struct("Layout", FIELDS, LayoutVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::layout::Layout;
    use crate::sketchbook::VarId;
    use std::str::FromStr;

    #[test]
    fn test_layout_serde() {
        let mut layout = Layout::new_empty("layout_name").unwrap();
        layout.add_node(VarId::new("v1").unwrap(), 1., 1.).unwrap();

        // Serialization
        let layout_serialized = serde_json::to_string(&layout).unwrap();
        // this cant fail due to order of nodes, since we only have one
        assert_eq!(
            "{\"name\":\"layout_name\",\"nodes\":{\"v1\":{\"position\":[1.0,1.0]}}}".to_string(),
            layout_serialized
        );
        assert_eq!(layout.to_string(), layout_serialized);

        // Deserialization
        let layout_v2: Layout = serde_json::from_str(&layout_serialized).unwrap();
        assert_eq!(layout, layout_v2);
        assert_eq!(layout, Layout::from_str(&layout_serialized).unwrap());
    }
}
