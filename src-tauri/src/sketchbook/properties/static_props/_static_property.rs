use crate::sketchbook::ids::{UninterpretedFnId, VarId};
use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::properties::static_props::*;
use crate::sketchbook::properties::FirstOrderFormula;
use crate::sketchbook::utils::assert_name_valid;
use serde::{Deserialize, Serialize};

/// A typesafe representation of various kinds of static properties.
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
    pub fn mk_regulation_essential(
        name: &str,
        input: Option<VarId>,
        target: Option<VarId>,
        value: Essentiality,
    ) -> Result<StatProperty, String> {
        let property = RegulationEssential {
            input,
            target,
            value,
            context: None,
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::RegulationEssential(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an update function is essential
    /// in a certain context.
    pub fn mk_regulation_essential_context(
        name: &str,
        input: Option<VarId>,
        target: Option<VarId>,
        value: Essentiality,
        context: String,
    ) -> Result<StatProperty, String> {
        let property = RegulationEssential {
            input,
            target,
            value,
            context: Some(context),
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::RegulationEssentialContext(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an update function is monotonic.
    pub fn mk_regulation_monotonic(
        name: &str,
        input: Option<VarId>,
        target: Option<VarId>,
        value: Monotonicity,
    ) -> Result<StatProperty, String> {
        let property = RegulationMonotonic {
            input,
            target,
            value,
            context: None,
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::RegulationMonotonic(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an update function is monotonic
    /// in a certain context.
    pub fn mk_regulation_monotonic_context(
        name: &str,
        input: Option<VarId>,
        target: Option<VarId>,
        value: Monotonicity,
        context: String,
    ) -> Result<StatProperty, String> {
        let property = RegulationMonotonic {
            input,
            target,
            value,
            context: Some(context),
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::RegulationMonotonicContext(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is essential.
    pub fn mk_fn_input_essential(
        name: &str,
        input_index: Option<usize>,
        target: Option<UninterpretedFnId>,
        value: Essentiality,
    ) -> Result<StatProperty, String> {
        let property = FnInputEssential {
            input_index,
            target,
            value,
            context: None,
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::FnInputEssentialContext(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is essential in a certain context.
    pub fn mk_fn_input_essential_context(
        name: &str,
        input_index: Option<usize>,
        target: Option<UninterpretedFnId>,
        value: Essentiality,
        context: String,
    ) -> Result<StatProperty, String> {
        let property = FnInputEssential {
            input_index,
            target,
            value,
            context: Some(context),
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::FnInputEssentialContext(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is monotonic.
    pub fn mk_fn_input_monotonic(
        name: &str,
        input_index: Option<usize>,
        target: Option<UninterpretedFnId>,
        value: Monotonicity,
    ) -> Result<StatProperty, String> {
        let property = FnInputMonotonic {
            input_index,
            target,
            value,
            context: None,
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::FnInputMonotonic(property),
        })
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is monotonic in a certain context.
    pub fn mk_fn_input_monotonic_context(
        name: &str,
        input_index: Option<usize>,
        target: Option<UninterpretedFnId>,
        value: Monotonicity,
        context: String,
    ) -> Result<StatProperty, String> {
        let property = FnInputMonotonic {
            input_index,
            target,
            value,
            context: Some(context),
        };
        Ok(StatProperty {
            name: name.to_string(),
            variant: StatPropertyType::FnInputMonotonicContext(property),
        })
    }

    /// Create default `StatProperty` instance of specified variant.
    pub fn default(variant: SimpleStatPropertyType) -> StatProperty {
        match variant {
            SimpleStatPropertyType::GenericStatProp => Self::default_generic(),
            SimpleStatPropertyType::RegulationEssential => Self::default_regulation_essential(),
            SimpleStatPropertyType::RegulationEssentialContext => {
                Self::default_regulation_essential_context()
            }
            SimpleStatPropertyType::RegulationMonotonic => Self::default_regulation_monotonic(),
            SimpleStatPropertyType::RegulationMonotonicContext => {
                Self::default_regulation_monotonic_context()
            }
            SimpleStatPropertyType::FnInputEssential => Self::default_fn_input_essential(),
            SimpleStatPropertyType::FnInputEssentialContext => {
                Self::default_fn_input_essential_context()
            }
            SimpleStatPropertyType::FnInputMonotonic => Self::default_fn_input_monotonic(),
            SimpleStatPropertyType::FnInputMonotonicContext => {
                Self::default_fn_input_monotonic_context()
            }
        }
    }

    /// Create default "generic" `StatProperty` instance, representing "true" formula.
    pub fn default_generic() -> StatProperty {
        Self::mk_generic("Generic static property", "true").unwrap()
    }

    /// Create default `StatProperty` instance for regulation essentiality (with empty `input` and
    /// `target` fields and `Unknown` essentiality).
    pub fn default_regulation_essential() -> StatProperty {
        Self::mk_regulation_essential("Regulation essential", None, None, Essentiality::Unknown)
            .unwrap()
    }

    /// Create default `StatProperty` instance for regulation essentiality in a context
    /// (with empty `input`, `target`, and `context` fields and `Unknown` essentiality).
    pub fn default_regulation_essential_context() -> StatProperty {
        Self::mk_regulation_essential_context(
            "Regulation essential",
            None,
            None,
            Essentiality::Unknown,
            String::new(),
        )
        .unwrap()
    }

    /// Create default `StatProperty` instance for regulation monotonicity (with empty `input` and
    /// `target` fields and `Unknown` monotonicity).
    pub fn default_regulation_monotonic() -> StatProperty {
        Self::mk_regulation_monotonic("Regulation monotonic", None, None, Monotonicity::Unknown)
            .unwrap()
    }

    /// Create default `StatProperty` instance for regulation monotonicity in a context
    /// (with empty `input`, `target`, and `context` fields and `Unknown` monotonicity).
    pub fn default_regulation_monotonic_context() -> StatProperty {
        Self::mk_regulation_monotonic_context(
            "Regulation monotonic",
            None,
            None,
            Monotonicity::Unknown,
            String::new(),
        )
        .unwrap()
    }

    /// Create default `StatProperty` instance for function input essentiality (with empty `input`
    /// and `target` fields and `Unknown` essentiality).
    pub fn default_fn_input_essential() -> StatProperty {
        Self::mk_fn_input_essential(
            "Function input essential",
            None,
            None,
            Essentiality::Unknown,
        )
        .unwrap()
    }

    /// Create default `StatProperty` instance for function input essentiality in a context
    /// (with empty `input`, `target`, and `context` fields and `Unknown` essentiality).
    pub fn default_fn_input_essential_context() -> StatProperty {
        Self::mk_fn_input_essential_context(
            "Function input essential",
            None,
            None,
            Essentiality::Unknown,
            String::new(),
        )
        .unwrap()
    }

    /// Create default `StatProperty` instance for function input monotonicity (with empty `input`
    /// and `target` fields and `Unknown` monotonicity).
    pub fn default_fn_input_monotonic() -> StatProperty {
        Self::mk_fn_input_monotonic(
            "Function input monotonic",
            None,
            None,
            Monotonicity::Unknown,
        )
        .unwrap()
    }

    /// Create default `StatProperty` instance for function input monotonicity in a context
    /// (with empty `input`, `target`, and `context` fields and `Unknown` monotonicity).
    pub fn default_fn_input_monotonic_context() -> StatProperty {
        Self::mk_fn_input_monotonic_context(
            "Function input monotonic",
            None,
            None,
            Monotonicity::Unknown,
            String::new(),
        )
        .unwrap()
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
        let new_var = Some(new_var);
        match &mut self.variant {
            StatPropertyType::RegulationMonotonic(prop) => prop.input = new_var,
            StatPropertyType::RegulationMonotonicContext(prop) => prop.input = new_var,
            StatPropertyType::RegulationEssential(prop) => prop.input = new_var,
            StatPropertyType::RegulationEssentialContext(prop) => prop.input = new_var,
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
        let new_idx = Some(new_idx);
        match &mut self.variant {
            StatPropertyType::FnInputEssential(prop) => prop.input_index = new_idx,
            StatPropertyType::FnInputEssentialContext(prop) => prop.input_index = new_idx,
            StatPropertyType::FnInputMonotonic(prop) => prop.input_index = new_idx,
            StatPropertyType::FnInputMonotonicContext(prop) => prop.input_index = new_idx,
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
        let new_target = Some(new_target);
        match &mut self.variant {
            StatPropertyType::FnInputEssential(prop) => prop.target = new_target,
            StatPropertyType::FnInputMonotonic(prop) => prop.target = new_target,
            StatPropertyType::FnInputEssentialContext(prop) => prop.target = new_target,
            StatPropertyType::FnInputMonotonicContext(prop) => prop.target = new_target,
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
        let new_target = Some(new_target);
        match &mut self.variant {
            StatPropertyType::RegulationEssential(prop) => prop.target = new_target,
            StatPropertyType::RegulationEssentialContext(prop) => prop.target = new_target,
            StatPropertyType::RegulationMonotonic(prop) => prop.target = new_target,
            StatPropertyType::RegulationMonotonicContext(prop) => prop.target = new_target,
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
            StatPropertyType::RegulationMonotonic(prop) => prop.value = monotonicity,
            StatPropertyType::FnInputMonotonicContext(prop) => prop.value = monotonicity,
            StatPropertyType::RegulationMonotonicContext(prop) => prop.value = monotonicity,
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
            StatPropertyType::RegulationEssential(prop) => prop.value = essentiality,
            StatPropertyType::FnInputEssentialContext(prop) => prop.value = essentiality,
            StatPropertyType::RegulationEssentialContext(prop) => prop.value = essentiality,
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
    pub fn set_context(&mut self, context: String) -> Result<(), String> {
        let context = Some(context);
        match &mut self.variant {
            StatPropertyType::FnInputEssentialContext(prop) => prop.context = context,
            StatPropertyType::FnInputMonotonicContext(prop) => prop.context = context,
            StatPropertyType::RegulationEssentialContext(prop) => prop.context = context,
            StatPropertyType::RegulationMonotonicContext(prop) => prop.context = context,
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
