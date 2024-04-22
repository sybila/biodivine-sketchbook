use crate::sketchbook::layout::Layout;
use crate::sketchbook::model::{ModelState, UninterpretedFn, UpdateFn, Variable};
use crate::sketchbook::utils::{parse_map_keys, stringify_and_order_keys};

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
        let mut state = serializer.serialize_struct("ModelState", 5)?;

        // Serialize all the HashMap fields with non-String keys as a HashMap with String keys
        // Serialize other fields as they are

        let variables_map = stringify_and_order_keys(&self.variables);
        state.serialize_field("variables", &variables_map)?;

        let update_fns_map = stringify_and_order_keys(&self.update_fns);
        state.serialize_field("update_fns", &update_fns_map)?;

        let uninterpreted_fns_map = stringify_and_order_keys(&self.uninterpreted_fns);
        state.serialize_field("uninterpreted_fns", &uninterpreted_fns_map)?;

        state.serialize_field("regulations", &self.regulations)?;

        let layouts_map = stringify_and_order_keys(&self.layouts);
        state.serialize_field("layouts", &layouts_map)?;

        state.serialize_field("placeholder_variables", &self.placeholder_variables)?;

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
            UpdateFns,
            UninterpretedFns,
            Layouts,
            PlaceholderVariables,
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
                        formatter.write_str(
                            "`variables`, `update_fns`, `uninterpreted_fns`, `regulations`, `layouts`, or `placeholder_variables`",
                        )
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "variables" => Ok(Field::Variables),
                            "regulations" => Ok(Field::Regulations),
                            "update_fns" => Ok(Field::UpdateFns),
                            "uninterpreted_fns" => Ok(Field::UninterpretedFns),
                            "layouts" => Ok(Field::Layouts),
                            "placeholder_variables" => Ok(Field::PlaceholderVariables),
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
                let mut update_fns = None;
                let mut uninterpreted_fns = None;
                let mut layouts = None;
                let mut placeholder_variables = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Variables => {
                            if variables.is_some() {
                                return Err(de::Error::duplicate_field("variables"));
                            }
                            let v: HashMap<String, Variable> = map.next_value()?;
                            variables = Some(parse_map_keys(v).map_err(de::Error::custom)?);
                        }
                        Field::UpdateFns => {
                            if update_fns.is_some() {
                                return Err(de::Error::duplicate_field("update_fns"));
                            }
                            let u: HashMap<String, UpdateFn> = map.next_value()?;
                            update_fns = Some(parse_map_keys(u).map_err(de::Error::custom)?);
                        }
                        Field::UninterpretedFns => {
                            if uninterpreted_fns.is_some() {
                                return Err(de::Error::duplicate_field("uninterpreted_fns"));
                            }
                            let u: HashMap<String, UninterpretedFn> = map.next_value()?;
                            uninterpreted_fns = Some(parse_map_keys(u).map_err(de::Error::custom)?);
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
                            layouts = Some(parse_map_keys(l).map_err(de::Error::custom)?);
                        }
                        Field::PlaceholderVariables => {
                            if placeholder_variables.is_some() {
                                return Err(de::Error::duplicate_field("placeholder_variables"));
                            }
                            placeholder_variables = Some(map.next_value()?);
                        }
                    }
                }

                let variables = variables.ok_or_else(|| de::Error::missing_field("variables"))?;
                let regulations =
                    regulations.ok_or_else(|| de::Error::missing_field("regulations"))?;
                let uninterpreted_fns = uninterpreted_fns
                    .ok_or_else(|| de::Error::missing_field("uninterpreted_fns"))?;
                let update_fns =
                    update_fns.ok_or_else(|| de::Error::missing_field("update_fns"))?;
                let layouts = layouts.ok_or_else(|| de::Error::missing_field("layouts"))?;
                let placeholder_variables = placeholder_variables
                    .ok_or_else(|| de::Error::missing_field("placeholder_variables"))?;
                Ok(ModelState {
                    variables,
                    regulations,
                    update_fns,
                    uninterpreted_fns,
                    layouts,
                    placeholder_variables,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "variables",
            "regulations",
            "update_fns",
            "uninterpreted_fns",
            "layouts",
            "placeholder_variables",
        ];
        deserializer.deserialize_struct("ModelState", FIELDS, ModelStateVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::ids::VarId;
    use crate::sketchbook::model::ModelState;
    use crate::sketchbook::JsonSerde;

    #[test]
    fn test_model_state_serde() {
        // test on very simple `ModelState` with one var and no regulations
        let mut model = ModelState::new_empty();
        let var_id = VarId::new("a").unwrap();
        model.add_var(var_id, "a").unwrap();

        // Serialization (and `to_string`)
        let model_serialized = serde_json::to_string(&model).unwrap();
        // this cant fail due to order of vars, since we only have one
        assert_eq!(
            "{\"variables\":{\"a\":{\"name\":\"a\"}},\"update_fns\":{\"a\":{\"expression\":\"\",\"tree\":null}},\"uninterpreted_fns\":{},\"regulations\":[],\"layouts\":{\"default\":{\"name\":\"default\",\"nodes\":{\"a\":{\"position\":[0.0,0.0]}}}},\"placeholder_variables\":[]}".to_string(),
            model_serialized
        );
        assert_eq!(model.to_json_str(), model_serialized);

        // Deserialization (and `from_str`)
        let model_v2 = ModelState::from_json_str(&model_serialized).unwrap();
        assert_eq!(model, model_v2);
        assert_eq!(model, ModelState::from_json_str(&model_serialized).unwrap());
    }
}
