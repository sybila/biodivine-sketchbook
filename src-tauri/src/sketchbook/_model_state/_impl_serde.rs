use crate::sketchbook::{Layout, LayoutId, ModelState, VarId, Variable};

use serde::de::{self, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;

impl Serialize for ModelState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ModelState", 3)?;

        // Serialize `variables` as a map with String keys
        let variables_map: HashMap<String, &Variable> = self
            .variables
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        state.serialize_field("variables", &variables_map)?;

        // Serialize `regulations` as is
        state.serialize_field("regulations", &self.regulations)?;

        // Serialize `layouts` as a map with String keys
        let layouts_map: HashMap<String, &Layout> = self
            .layouts
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        state.serialize_field("layouts", &layouts_map)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for ModelState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Variables,
            Regulations,
            Layouts,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`variables`, `regulations`, or `layouts`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "variables" => Ok(Field::Variables),
                            "regulations" => Ok(Field::Regulations),
                            "layouts" => Ok(Field::Layouts),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ModelStateVisitor;

        impl<'de> Visitor<'de> for ModelStateVisitor {
            type Value = ModelState;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ModelState")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ModelState, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut variables = None;
                let mut regulations = None;
                let mut layouts = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Variables => {
                            if variables.is_some() {
                                return Err(de::Error::duplicate_field("variables"));
                            }
                            let v: HashMap<String, Variable> = map.next_value()?;
                            variables = Some(
                                v.into_iter()
                                    .map(|(k, v)| {
                                        k.parse::<VarId>()
                                            .map_err(de::Error::custom)
                                            .map(|k_parsed| (k_parsed, v))
                                    })
                                    .collect::<Result<HashMap<VarId, Variable>, _>>()?,
                            );
                        }
                        Field::Regulations => {
                            if regulations.is_some() {
                                return Err(de::Error::duplicate_field("regulations"));
                            }
                            regulations = Some(map.next_value()?);
                        }
                        Field::Layouts => {
                            if layouts.is_some() {
                                return Err(de::Error::duplicate_field("layouts"));
                            }
                            let l: HashMap<String, Layout> = map.next_value()?;
                            layouts = Some(
                                l.into_iter()
                                    .map(|(k, v)| {
                                        k.parse::<LayoutId>()
                                            .map_err(de::Error::custom)
                                            .map(|k_parsed| (k_parsed, v))
                                    })
                                    .collect::<Result<HashMap<LayoutId, Layout>, _>>()?,
                            );
                        }
                    }
                }

                let variables = variables.ok_or_else(|| de::Error::missing_field("variables"))?;
                let regulations =
                    regulations.ok_or_else(|| de::Error::missing_field("regulations"))?;
                let layouts = layouts.ok_or_else(|| de::Error::missing_field("layouts"))?;
                Ok(ModelState {
                    variables,
                    regulations,
                    layouts,
                })
            }
        }

        const FIELDS: &[&str] = &["variables", "regulations", "layouts"];
        deserializer.deserialize_struct("ModelState", FIELDS, ModelStateVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::{ModelState, VarId};
    use std::str::FromStr;

    #[test]
    fn test_model_state_serde() {
        // test on very simple `ModelState` with one var and no regulations
        let mut model = ModelState::new();
        let var_id = VarId::new("a").unwrap();
        model.add_var(var_id, "a").unwrap();

        // Serialization (and `to_string`)
        let model_serialized = serde_json::to_string(&model).unwrap();
        // this cant fail due to order of vars, since we only have one
        assert_eq!(
            "{\"variables\":{\"a\":{\"name\":\"a\"}},\"regulations\":[],\"layouts\":{\"default_layout\":{\"name\":\"default_layout\",\"nodes\":{\"a\":{\"position\":[0.0,0.0]}}}}}".to_string(),
            model_serialized
        );
        assert_eq!(model.to_string(), model_serialized);

        // Deserialization (and `from_str`)
        let model_v2: ModelState = serde_json::from_str(&model_serialized).unwrap();
        assert_eq!(model, model_v2);
        assert_eq!(model, ModelState::from_str(&model_serialized).unwrap());
    }

    #[test]
    fn test_from_to_string() {
        let mut model = ModelState::new();
        let var_id = VarId::new("a").unwrap();
        model.add_var(var_id, "a").unwrap();
        model.add_regulation_by_str("a -> a").unwrap();

        // To string
        let model_string = model.to_string();
        assert_eq!(
            "{\"variables\":{\"a\":{\"name\":\"a\"}},\"regulations\":[{\"regulator\":{\"id\":{\"id\":\"a\"}},\"target\":{\"id\":{\"id\":\"a\"}},\"observable\":\"True\",\"regulation_sign\":\"Activation\"}],\"layouts\":{\"default_layout\":{\"name\":\"default_layout\",\"nodes\":{\"a\":{\"position\":[0.0,0.0]}}}}}".to_string(),
            model_string
        );

        // From String
        let model_v2 = ModelState::from_str(&model_string).unwrap();
        assert_eq!(model, model_v2);
    }
}
