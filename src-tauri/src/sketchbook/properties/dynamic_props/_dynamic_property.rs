use crate::sketchbook::ids::{DatasetId, ObservationId};
use crate::sketchbook::properties::dynamic_props::*;
use crate::sketchbook::utils::assert_name_valid;
use serde::{Deserialize, Serialize};

/// A typesafe representation wrapping various kinds of dynamic properties.
/// Each property has a `name` and field `variant` encompassing inner data.
///
/// The formula that will be internally created (usually, apart from generic variant) depends on
/// particular type of the property - there are multiple `variants` of properties, each carrying
/// its own different metadata that are later used to build the formula.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DynProperty {
    name: String,
    variant: DynPropertyType,
}

/// Creating dynamic properties.
impl DynProperty {
    /// Create "generic" `DynProperty` instance directly from a formula, which must be in a
    /// correct format (which is verified).
    pub fn mk_generic(name: &str, raw_formula: &str) -> Result<DynProperty, String> {
        let property = GenericDynProp {
            raw_formula: raw_formula.to_string(),
            processed_formula: HctlFormula::try_from_str(raw_formula)?,
        };
        Ok(DynProperty {
            name: name.to_string(),
            variant: DynPropertyType::GenericDynProp(property),
        })
    }

    /// Create `DynProperty` instance describing existence of a fixed point corresponding to
    /// a given observation.
    pub fn mk_fixed_point(
        name: &str,
        dataset: Option<DatasetId>,
        observation: Option<ObservationId>,
    ) -> Result<DynProperty, String> {
        let property = ExistsFixedPoint {
            dataset,
            observation,
        };
        Ok(DynProperty {
            name: name.to_string(),
            variant: DynPropertyType::ExistsFixedPoint(property),
        })
    }

    /// Create `DynProperty` instance describing existence of a trap space corresponding to
    /// a given observation.
    pub fn mk_trap_space(
        name: &str,
        dataset: Option<DatasetId>,
        observation: Option<ObservationId>,
        minimal: bool,
        nonpercolable: bool,
    ) -> Result<DynProperty, String> {
        let property = ExistsTrapSpace {
            dataset,
            observation,
            minimal,
            nonpercolable,
        };
        Ok(DynProperty {
            name: name.to_string(),
            variant: DynPropertyType::ExistsTrapSpace(property),
        })
    }

    /// Create `DynProperty` instance describing existence of a trajectory corresponding to
    /// observations from a given observation (in the given order).
    pub fn mk_trajectory(name: &str, dataset: Option<DatasetId>) -> Result<DynProperty, String> {
        let property = ExistsTrajectory { dataset };
        Ok(DynProperty {
            name: name.to_string(),
            variant: DynPropertyType::ExistsTrajectory(property),
        })
    }

    /// Create `DynProperty` instance describing the number of existing attractors.
    pub fn mk_attractor_count(
        name: &str,
        minimal: usize,
        maximal: usize,
    ) -> Result<DynProperty, String> {
        let property = AttractorCount { minimal, maximal };
        Ok(DynProperty {
            name: name.to_string(),
            variant: DynPropertyType::AttractorCount(property),
        })
    }

    /// Create `DynProperty` instance describing the existence of an attractor corresponding to
    /// a corresponding dataset, or some specific observation in it.
    pub fn mk_has_attractor(
        name: &str,
        dataset: Option<DatasetId>,
        observation: Option<ObservationId>,
    ) -> Result<DynProperty, String> {
        let property = HasAttractor {
            dataset,
            observation,
        };
        Ok(DynProperty {
            name: name.to_string(),
            variant: DynPropertyType::HasAttractor(property),
        })
    }

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
        Self::mk_generic("Generic property", "true").unwrap()
    }

    /// Create default `DynProperty` instance for the existence of a fixed point, with empty
    /// `dataset` and `observation` fields.
    pub fn default_fixed_point() -> DynProperty {
        Self::mk_fixed_point("Fixed-point property", None, None).unwrap()
    }

    /// Create default `DynProperty` instance for the existence of a trap space, with empty
    /// `dataset` and `observation` fields, and all flags set to false.
    pub fn default_trap_space() -> DynProperty {
        Self::mk_trap_space("Trap-space property", None, None, false, false).unwrap()
    }

    /// Create default `DynProperty` instance for the existence of a trajectory, with an empty
    /// `dataset`field.
    pub fn default_trajectory() -> DynProperty {
        Self::mk_trajectory("Trajectory property", None).unwrap()
    }

    /// Create default `DynProperty` instance for the number of existing attractors, with default
    /// count being 1.
    pub fn default_attractor_count() -> DynProperty {
        Self::mk_attractor_count("Attractor-count property", 1, 1).unwrap()
    }

    /// Create default `DynProperty` instance for the existence of an attractor with empty
    /// `dataset` and `observation` fields.
    pub fn default_has_attractor() -> DynProperty {
        Self::mk_has_attractor("Attractor property", None, None).unwrap()
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
            // first check everything is valid, then update fields
            let parsed_formula = HctlFormula::try_from_str(new_formula)?;
            prop.processed_formula = parsed_formula;
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

    /// Get property's variant with all the underlying data.
    pub fn get_prop_data(&self) -> &DynPropertyType {
        &self.variant
    }
}
