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
///
/// todo: decide which class will be responsible for the encoding of predefined properties to formulas (probably PropertyManager).
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
    /// Set property's name.
    pub fn set_name(&mut self, new_name: &str) -> Result<(), String> {
        assert_name_valid(new_name)?;
        self.name = new_name.to_string();
        Ok(())
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
