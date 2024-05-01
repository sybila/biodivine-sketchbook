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
        dataset: DatasetId,
        observation: ObservationId,
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
        dataset: DatasetId,
        observation: ObservationId,
        minimal: bool,
        non_percolable: bool,
    ) -> Result<DynProperty, String> {
        let property = ExistsTrapSpace {
            dataset,
            observation,
            minimal,
            non_percolable,
        };
        Ok(DynProperty {
            name: name.to_string(),
            variant: DynPropertyType::ExistsTrapSpace(property),
        })
    }

    /// Create `DynProperty` instance describing existence of a trajectory corresponding to
    /// observations from a given observation (in the given order).
    pub fn mk_trajectory(name: &str, dataset: DatasetId) -> Result<DynProperty, String> {
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
        dataset: DatasetId,
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
        match &mut self.variant {
            DynPropertyType::ExistsFixedPoint(prop) => prop.observation = new_obs,
            DynPropertyType::ExistsTrapSpace(prop) => prop.observation = new_obs,
            DynPropertyType::HasAttractor(prop) => prop.observation = Some(new_obs),
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
        non_percolable: bool,
    ) -> Result<(), String> {
        if let DynPropertyType::ExistsTrapSpace(prop) = &mut self.variant {
            prop.minimal = is_minimal;
            prop.non_percolable = non_percolable;
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
