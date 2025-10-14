use super::_wild_card_props::process_wild_card_props;
use crate::sketchbook::ids::{DatasetId, ObservationId};
use crate::sketchbook::properties::dynamic_props::*;
use crate::sketchbook::utils::assert_name_valid;
use serde::{Deserialize, Serialize};

/// A typesafe representation wrapping various kinds of dynamic properties.
/// Each property has a `name` and field `variant` encompassing inner data.
/// It can also be annotated using a string `annotation` field.
///
/// Different kinds of properties can be evaluated using different algorithms. The
/// standard way is to build an HCTL formula based on the property type and its data,
/// and use model checking to evaluate it.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DynProperty {
    name: String,
    annotation: String,
    variant: DynPropertyType,
}

/// Creating dynamic properties.
impl DynProperty {
    /// **(internal)** Shorthand to create a property given fully prepared internal
    /// `DynPropertyType` data and name. Annotation is left empty for now.
    fn new_raw(name: &str, variant: DynPropertyType) -> DynProperty {
        DynProperty {
            name: name.to_string(),
            annotation: String::new(),
            variant,
        }
    }

    /// Update the `annotation` property.
    pub fn with_annotation(mut self, annotation: &str) -> Self {
        self.annotation = annotation.to_string();
        self
    }

    /// Create new "generic" `DynProperty` instance directly from a formula, which must be
    /// in a correct format (the general syntax is checked). Annotation is left empty for now.
    pub fn try_mk_generic(name: &str, raw_formula: &str) -> Result<DynProperty, String> {
        // first process potential wild card propositions
        // this collects all the wild cards, and modifies them to adhere to internal syntax rules
        let (modified_formula, wild_cards) = process_wild_card_props(raw_formula)?;
        let property = GenericDynProp {
            raw_formula: raw_formula.to_string(),
            processed_formula: HctlFormula::try_from_str(&modified_formula)?,
            wild_cards,
        };
        let variant = DynPropertyType::GenericDynProp(property);
        Ok(Self::new_raw(name, variant))
    }

    /// Create new `DynProperty` instance describing existence of a fixed point corresponding
    /// to a given observation. Dataset and observation can be left out, but they have to
    /// be specified before the inference. Annotation is left empty for now.
    pub fn mk_fixed_point(
        name: &str,
        dataset: Option<DatasetId>,
        observation: Option<ObservationId>,
    ) -> DynProperty {
        let property = ExistsFixedPoint {
            dataset,
            observation,
        };
        let variant = DynPropertyType::ExistsFixedPoint(property);
        Self::new_raw(name, variant)
    }

    /// Create new `DynProperty` instance describing existence of a trap space corresponding
    /// to a given observation. Optionally, the trap space might be required to be minimal or
    /// non-percolable. Annotation is left empty for now.
    pub fn mk_trap_space(
        name: &str,
        dataset: Option<DatasetId>,
        observation: Option<ObservationId>,
        minimal: bool,
        nonpercolable: bool,
    ) -> DynProperty {
        let property = ExistsTrapSpace {
            dataset,
            observation,
            minimal,
            nonpercolable,
        };
        let variant = DynPropertyType::ExistsTrapSpace(property);
        Self::new_raw(name, variant)
    }

    /// Create `DynProperty` instance describing existence of a trajectory corresponding to
    /// observations from a given observation (in the given order). Annotation is left empty
    /// for now.
    pub fn mk_trajectory(name: &str, dataset: Option<DatasetId>) -> DynProperty {
        let property = ExistsTrajectory { dataset };
        let variant = DynPropertyType::ExistsTrajectory(property);
        Self::new_raw(name, variant)
    }

    /// Create `DynProperty` instance describing the number of existing attractors.\
    /// Annotation is left empty for now.
    pub fn try_mk_attractor_count(
        name: &str,
        minimal: usize,
        maximal: usize,
    ) -> Result<DynProperty, String> {
        if minimal > maximal {
            return Err("`minimal` attractor count cannot be larger than `maximal`.".to_string());
        }
        if minimal == 0 || maximal == 0 {
            return Err("Attractor count must be larger than 0.".to_string());
        }
        let property = AttractorCount { minimal, maximal };
        let variant = DynPropertyType::AttractorCount(property);
        Ok(Self::new_raw(name, variant))
    }

    /// Create `DynProperty` instance describing the existence of an attractor corresponding
    /// to a corresponding dataset, or some specific observation in it. Annotation is left
    /// empty for now.
    pub fn mk_has_attractor(
        name: &str,
        dataset: Option<DatasetId>,
        observation: Option<ObservationId>,
    ) -> DynProperty {
        let property = HasAttractor {
            dataset,
            observation,
        };
        let variant = DynPropertyType::HasAttractor(property);
        Self::new_raw(name, variant)
    }

    /// Create default `DynProperty` instance of specified variant.
    pub fn default(variant: SimpleDynPropertyType) -> DynProperty {
        match variant {
            SimpleDynPropertyType::GenericDynProp => Self::default_generic(),
            SimpleDynPropertyType::ExistsFixedPoint => Self::default_fixed_point(),
            SimpleDynPropertyType::ExistsTrapSpace => Self::default_trap_space(),
            SimpleDynPropertyType::ExistsTrajectory => Self::default_trajectory(),
            SimpleDynPropertyType::AttractorCount => Self::default_attractor_count(),
            SimpleDynPropertyType::HasAttractor => Self::default_has_attractor(),
        }
    }

    /// Create default "generic" `DynProperty` instance, representing "true" formula.
    pub fn default_generic() -> DynProperty {
        Self::try_mk_generic("New generic dynamic property", "true").unwrap()
    }

    /// Create default `DynProperty` instance for the existence of a fixed point, with empty
    /// `dataset` and `observation` fields.
    pub fn default_fixed_point() -> DynProperty {
        Self::mk_fixed_point("New exist fixed points property", None, None)
    }

    /// Create default `DynProperty` instance for the existence of a trap space, with empty
    /// `dataset` and `observation` fields, and all flags set to false.
    pub fn default_trap_space() -> DynProperty {
        Self::mk_trap_space("New exist trap spaces property", None, None, false, false)
    }

    /// Create default `DynProperty` instance for the existence of a trajectory, with an empty
    /// `dataset`field.
    pub fn default_trajectory() -> DynProperty {
        Self::mk_trajectory("New exist trajectory property", None)
    }

    /// Create default `DynProperty` instance for the number of existing attractors, with default
    /// count being 1.
    pub fn default_attractor_count() -> DynProperty {
        Self::try_mk_attractor_count("New attractor count property", 1, 1).unwrap()
    }

    /// Create default `DynProperty` instance for the existence of an attractor with empty
    /// `dataset` and `observation` fields.
    pub fn default_has_attractor() -> DynProperty {
        Self::mk_has_attractor("New exist attractors property", None, None)
    }
}

/// Editing dynamic properties.
impl DynProperty {
    /// Update property's name.
    pub fn set_name(&mut self, new_name: &str) -> Result<(), String> {
        assert_name_valid(new_name)?;
        self.name = new_name.to_string();
        Ok(())
    }

    /// Set property's annotation string.
    pub fn set_annotation(&mut self, annotation: &str) {
        self.annotation = annotation.to_string()
    }

    /// Update property's sub-field `dataset` where applicable. If not applicable, return `Err`.
    pub fn set_dataset(&mut self, new_dataset: DatasetId) -> Result<(), String> {
        let new_dataset = Some(new_dataset);
        match &mut self.variant {
            DynPropertyType::ExistsFixedPoint(prop) => prop.dataset = new_dataset,
            DynPropertyType::ExistsTrapSpace(prop) => prop.dataset = new_dataset,
            DynPropertyType::ExistsTrajectory(prop) => prop.dataset = new_dataset,
            DynPropertyType::HasAttractor(prop) => prop.dataset = new_dataset,
            // Other cases do not have a dataset field
            other_variant => {
                return Err(format!(
                    "{other_variant:?} does not have a field `dataset`."
                ));
            }
        }
        Ok(())
    }

    /// Update property's sub-field `observation` where applicable. If not applicable, return `Err`.
    pub fn set_observation(&mut self, new_obs: ObservationId) -> Result<(), String> {
        let new_obs = Some(new_obs);
        match &mut self.variant {
            DynPropertyType::ExistsFixedPoint(prop) => prop.observation = new_obs,
            DynPropertyType::ExistsTrapSpace(prop) => prop.observation = new_obs,
            DynPropertyType::HasAttractor(prop) => prop.observation = new_obs,
            // Other cases do not have a observation field
            other_variant => {
                return Err(format!(
                    "{other_variant:?} does not have a field `observation`."
                ));
            }
        }
        Ok(())
    }

    /// Update generic property's formula. If not applicable (different variant), return `Err`.
    pub fn set_formula(&mut self, new_formula: &str) -> Result<(), String> {
        if let DynPropertyType::GenericDynProp(prop) = &mut self.variant {
            // first check everything is valid (including potential wild-card props), then update fields
            let (modified_formula, wild_cards) = process_wild_card_props(new_formula)?;
            let parsed_formula = HctlFormula::try_from_str(&modified_formula)?;
            prop.processed_formula = parsed_formula;
            prop.wild_cards = wild_cards;
            prop.raw_formula = new_formula.to_string();
            Ok(())
        } else {
            Err(format!(
                "{:?} does not have a formula to update.",
                self.variant
            ))
        }
    }

    /// Update property's sub-field `observation` to None where applicable. If not applicable,
    /// return `Err`.
    pub fn remove_observation(&mut self) -> Result<(), String> {
        if let DynPropertyType::HasAttractor(prop) = &mut self.variant {
            prop.observation = None;
            Ok(())
        } else {
            Err(format!(
                "{:?} does not have a observation to remove.",
                self.variant
            ))
        }
    }

    /// Update property's sub-fields, if the property is of `AttractorCount` variant.
    /// If not applicable, return `Err`.
    pub fn set_attr_count(&mut self, minimal: usize, maximal: usize) -> Result<(), String> {
        if let DynPropertyType::AttractorCount(prop) = &mut self.variant {
            prop.minimal = minimal;
            prop.maximal = maximal;
            Ok(())
        } else {
            Err(format!(
                "{:?} does not have a attractor count to update.",
                self.variant
            ))
        }
    }

    /// Update property's sub-fields, if the property is of `ExistsTrapSpace` variant.
    /// If not applicable, return `Err`.
    pub fn set_trap_space_details(
        &mut self,
        is_minimal: bool,
        nonpercolable: bool,
    ) -> Result<(), String> {
        if let DynPropertyType::ExistsTrapSpace(prop) = &mut self.variant {
            prop.minimal = is_minimal;
            prop.nonpercolable = nonpercolable;
            Ok(())
        } else {
            Err(format!(
                "{:?} does not have a trap space fields to update.",
                self.variant
            ))
        }
    }
}

/// Observing dynamic properties.
impl DynProperty {
    /// Get property's name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get annotation string.
    pub fn get_annotation(&self) -> &str {
        &self.annotation
    }

    /// Get property's variant with all the underlying data.
    pub fn get_prop_data(&self) -> &DynPropertyType {
        &self.variant
    }

    /// Get property's sub-field `dataset` if present. If not present, return `Err`.
    pub fn get_dataset(&self) -> Result<Option<DatasetId>, String> {
        match &self.variant {
            DynPropertyType::ExistsFixedPoint(prop) => Ok(prop.dataset.clone()),
            DynPropertyType::ExistsTrapSpace(prop) => Ok(prop.dataset.clone()),
            DynPropertyType::ExistsTrajectory(prop) => Ok(prop.dataset.clone()),
            DynPropertyType::HasAttractor(prop) => Ok(prop.dataset.clone()),
            // Other cases do not have a dataset field
            other_variant => Err(format!(
                "{other_variant:?} does not have a field `dataset`."
            )),
        }
    }

    /// Check that the property has all required fields filled out. That is just the `dataset`
    /// in most cases at the moment.
    /// If some of the required field is set to None, return error.
    pub fn assert_dataset_filled(&self) -> Result<(), String> {
        let missing_field_msg = "One of the required fields is not filled.";

        match &self.variant {
            DynPropertyType::GenericDynProp(_) => {} // no fields that can be None
            DynPropertyType::AttractorCount(_) => {} // no fields that can be None
            DynPropertyType::HasAttractor(p) => {
                // only dataset has to be filled, observation ID is optional
                if p.dataset.is_none() {
                    return Err(missing_field_msg.to_string());
                }
            }
            DynPropertyType::ExistsFixedPoint(p) => {
                // only dataset has to be filled, observation ID is optional
                if p.dataset.is_none() {
                    return Err(missing_field_msg.to_string());
                }
            }
            DynPropertyType::ExistsTrajectory(p) => {
                if p.dataset.is_none() {
                    return Err(missing_field_msg.to_string());
                }
            }
            DynPropertyType::ExistsTrapSpace(p) => {
                // only dataset has to be filled, observation ID is optional
                if p.dataset.is_none() {
                    return Err(missing_field_msg.to_string());
                }
            }
        }
        Ok(())
    }
}
