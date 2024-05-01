use crate::sketchbook::ids::{UninterpretedFnId, VarId};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::static_props::*;
use crate::sketchbook::properties::FirstOrderFormula;
use crate::sketchbook::utils::assert_name_valid;
use serde::{Deserialize, Serialize};

/// A typesafe representation of various kinds of dynamic properties.
/// Each property has a `name` and field `variant` encompassing inner data.
///
/// The formula that will be internally created (usually, apart from generic variant) depends on
/// particular type of the property - there are multiple `variants` of properties, each carrying
/// its own different metadata that are later used to build the formula.
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
    /// Set property's name.
    pub fn set_name(&mut self, new_name: &str) -> Result<(), String> {
        assert_name_valid(new_name)?;
        self.name = new_name.to_string();
        Ok(())
    }

    /// Update property's sub-field for input variable (of an update fn), where applicable.
    /// If not applicable, return `Err`.
    pub fn set_input_var(&mut self, new_var: VarId) -> Result<(), String> {
        match &mut self.variant {
            StatPropertyType::UpdateFnInputEssential(prop) => prop.input = new_var,
            StatPropertyType::UpdateFnInputMonotonic(prop) => prop.input = new_var,
            other_variant => {
                return Err(format!(
                    "{other_variant:?} does not have a field for input variable."
                ));
            }
        }
        Ok(())
    }

    /// Update property's sub-field for index of input (of an uninterpreted fn), where applicable.
    /// If not applicable, return `Err`.
    pub fn set_input_index(&mut self, new_idx: usize) -> Result<(), String> {
        match &mut self.variant {
            StatPropertyType::FnInputEssential(prop) => prop.input_index = new_idx,
            StatPropertyType::FnInputMonotonic(prop) => prop.input_index = new_idx,
            other_variant => {
                return Err(format!(
                    "{other_variant:?} does not have a field for input index."
                ));
            }
        }
        Ok(())
    }

    /// Update property's sub-field for target uninterpreted fn, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_target_fn(&mut self, new_target: UninterpretedFnId) -> Result<(), String> {
        match &mut self.variant {
            StatPropertyType::FnInputEssential(prop) => prop.target = new_target,
            StatPropertyType::FnInputMonotonic(prop) => prop.target = new_target,
            other_variant => {
                return Err(format!(
                    "{other_variant:?} does not have a field for target uninterpreted fn."
                ));
            }
        }
        Ok(())
    }

    /// Update property's sub-field for target variable, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_target_var(&mut self, new_target: VarId) -> Result<(), String> {
        match &mut self.variant {
            StatPropertyType::UpdateFnInputEssential(prop) => prop.target = new_target,
            StatPropertyType::UpdateFnInputMonotonic(prop) => prop.target = new_target,
            other_variant => {
                return Err(format!(
                    "{other_variant:?} does not have a field for target uninterpreted var."
                ));
            }
        }
        Ok(())
    }

    /// Update property's sub-field for monotonicity, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_monotonicity(&mut self, monotonicity: Monotonicity) -> Result<(), String> {
        match &mut self.variant {
            StatPropertyType::FnInputMonotonic(prop) => prop.value = monotonicity,
            StatPropertyType::UpdateFnInputMonotonic(prop) => prop.value = monotonicity,
            other_variant => {
                return Err(format!(
                    "{other_variant:?} does not have a field for monotonicity."
                ));
            }
        }
        Ok(())
    }

    /// Update property's sub-field for essentiality, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_essentiality(&mut self, essentiality: Essentiality) -> Result<(), String> {
        match &mut self.variant {
            StatPropertyType::FnInputEssential(prop) => prop.value = essentiality,
            StatPropertyType::UpdateFnInputEssential(prop) => prop.value = essentiality,
            other_variant => {
                return Err(format!(
                    "{other_variant:?} does not have a field for essentiality."
                ));
            }
        }
        Ok(())
    }

    /// Update property's sub-field for context, where applicable.
    /// If not applicable, return `Err`.
    pub fn set_context(&mut self, context: Option<String>) -> Result<(), String> {
        match &mut self.variant {
            StatPropertyType::FnInputEssential(prop) => prop.context = context,
            StatPropertyType::FnInputMonotonic(prop) => prop.context = context,
            StatPropertyType::UpdateFnInputEssential(prop) => prop.context = context,
            StatPropertyType::UpdateFnInputMonotonic(prop) => prop.context = context,
            other_variant => {
                return Err(format!(
                    "{other_variant:?} does not have a field for context."
                ));
            }
        }
        Ok(())
    }

    /// Update generic property's formula. If not applicable (different variant), return `Err`.
    pub fn set_formula(&mut self, new_formula: &str) -> Result<(), String> {
        if let StatPropertyType::GenericStatProp(prop) = &mut self.variant {
            // first check everything is valid, then update fields
            let parsed_formula = FirstOrderFormula::try_from_str(new_formula)?;
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
