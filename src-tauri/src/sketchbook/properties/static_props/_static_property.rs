use crate::sketchbook::ids::{UninterpretedFnId, VarId};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::static_props::*;
use crate::sketchbook::properties::FirstOrderFormula;
use serde::{Deserialize, Serialize};

/// A typesafe representation of various kinds of dynamic properties.
/// Each property has a `name` and a first-order `formula`. The formula depends on particular
/// type of the property - there are multiple kinds of static properties, each carrying
/// different metadata from which the formula is built.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StatProperty {
    name: String,
    variant: StatPropertyType,
}

/// Creating static properties.
impl StatProperty {
    /// Create "generic" `StatProperty` instance directly from a formula, which must be in a
    /// correct format (which is verified).
    ///
    /// TODO: `FirstOrderFormula` struct currently lacks syntax check
    pub fn mk_generic(name: &str, raw_formula: &str) -> Result<StatProperty, String> {
        let property = GenericStatProp {
            raw_formula: raw_formula.to_string(),
            processed_formula: FirstOrderFormula::try_from_str(raw_formula)?,
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::GenericStatProp(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an update function is essential.
    ///
    /// TODO: in future, generate the name automatically
    /// TODO: define how to handle context - should be parsed as FO formula?
    pub fn mk_update_fn_input_essential(
        name: &str,
        input: VarId,
        target: VarId,
        value: Essentiality,
        context: Option<String>,
    ) -> Result<StatProperty, String> {
        let property = UpdateFnInputEssential {
            input,
            target,
            value,
            context,
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::UpdateFnInputEssential(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an update function is monotonic.
    ///
    /// TODO: in future, generate the name automatically
    /// TODO: define how to handle context - should be parsed as FO formula?
    pub fn mk_update_fn_input_monotonic(
        name: &str,
        input: VarId,
        target: VarId,
        value: Monotonicity,
        context: Option<String>,
    ) -> Result<StatProperty, String> {
        let property = UpdateFnInputMonotonic {
            input,
            target,
            value,
            context,
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::UpdateFnInputMonotonic(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is essential.
    ///
    /// TODO: in future, generate the name automatically
    /// TODO: define how to handle context - should be parsed as FO formula?
    pub fn mk_fn_input_essential(
        name: &str,
        input_index: usize,
        target: UninterpretedFnId,
        value: Essentiality,
        context: Option<String>,
    ) -> Result<StatProperty, String> {
        let property = FnInputEssential {
            input_index,
            target,
            value,
            context,
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::FnInputEssential(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is monotonic.
    ///
    /// TODO: in future, generate the name automatically
    /// TODO: define how to handle context - should be parsed as FO formula?
    pub fn mk_fn_input_monotonic(
        name: &str,
        input_index: usize,
        target: UninterpretedFnId,
        value: Monotonicity,
        context: Option<String>,
    ) -> Result<StatProperty, String> {
        let property = FnInputMonotonic {
            input_index,
            target,
            value,
            context,
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::FnInputMonotonic(property),
        })
    }
}

/// Editing static properties.
impl StatProperty {
    // TODO
}

/// Observing static properties.
impl StatProperty {
    /// Get property's name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get property's variant with all the underlying data.
    pub fn get_prop_data(&self) -> &StatPropertyType {
        &self.variant
    }
}
