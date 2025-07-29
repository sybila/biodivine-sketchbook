use crate::sketchbook::ids::{StatPropertyId, UninterpretedFnId, VarId};
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
    annotation: String,
    variant: StatPropertyType,
}

/// Creating static properties.
impl StatProperty {
    /// **(internal)** Shorthand to create a named property given fully prepared internal
    /// `StatPropertyType` data. Annotation is left empty for now.
    fn new_raw(name: &str, variant: StatPropertyType) -> StatProperty {
        StatProperty {
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

    /// Create "generic" `StatProperty` instance directly from a formula, which must be in a
    /// correct format (general FOL syntax is checked). Annotation is left empty for now.
    pub fn try_mk_generic(name: &str, raw_formula: &str) -> Result<StatProperty, String> {
        let property = GenericStatProp {
            raw_formula: raw_formula.to_string(),
            processed_formula: FirstOrderFormula::try_from_str(raw_formula)?,
        };
        let variant = StatPropertyType::GenericStatProp(property);
        Ok(Self::new_raw(name, variant))
    }

    /// Create `StatProperty` instance describing that an input of an update function is
    /// essential. Annotation is left empty for now.
    pub fn mk_regulation_essential(
        name: &str,
        input: Option<VarId>,
        target: Option<VarId>,
        value: Essentiality,
    ) -> StatProperty {
        let property = RegulationEssential {
            input,
            target,
            value,
            context: None,
        };
        let variant = StatPropertyType::RegulationEssential(property);
        Self::new_raw(name, variant)
    }

    /// Create `StatProperty` instance describing that an input of an update function is
    /// essential in a certain context. Annotation is left empty for now.
    pub fn mk_regulation_essential_context(
        name: &str,
        input: Option<VarId>,
        target: Option<VarId>,
        value: Essentiality,
        context: String,
    ) -> StatProperty {
        let property = RegulationEssential {
            input,
            target,
            value,
            context: Some(context),
        };
        let variant = StatPropertyType::RegulationEssentialContext(property);
        Self::new_raw(name, variant)
    }

    /// Create `StatProperty` instance describing that an input of an update function is
    /// monotonic. Annotation is left empty for now.
    pub fn mk_regulation_monotonic(
        name: &str,
        input: Option<VarId>,
        target: Option<VarId>,
        value: Monotonicity,
    ) -> StatProperty {
        let property = RegulationMonotonic {
            input,
            target,
            value,
            context: None,
        };
        let variant = StatPropertyType::RegulationMonotonic(property);
        Self::new_raw(name, variant)
    }

    /// Create `StatProperty` instance describing that an input of an update function is
    /// monotonic in a certain context. Annotation is left empty for now.
    pub fn mk_regulation_monotonic_context(
        name: &str,
        input: Option<VarId>,
        target: Option<VarId>,
        value: Monotonicity,
        context: String,
    ) -> StatProperty {
        let property = RegulationMonotonic {
            input,
            target,
            value,
            context: Some(context),
        };
        let variant = StatPropertyType::RegulationMonotonicContext(property);
        Self::new_raw(name, variant)
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is essential. Annotation is left empty for now.
    pub fn mk_fn_input_essential(
        name: &str,
        input_index: Option<usize>,
        target: Option<UninterpretedFnId>,
        value: Essentiality,
    ) -> StatProperty {
        let property = FnInputEssential {
            input_index,
            target,
            value,
            context: None,
        };
        let variant = StatPropertyType::FnInputEssential(property);
        Self::new_raw(name, variant)
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is essential in a certain context. Annotation is left empty for now.
    pub fn mk_fn_input_essential_context(
        name: &str,
        input_index: Option<usize>,
        target: Option<UninterpretedFnId>,
        value: Essentiality,
        context: String,
    ) -> StatProperty {
        let property = FnInputEssential {
            input_index,
            target,
            value,
            context: Some(context),
        };
        let variant = StatPropertyType::FnInputEssentialContext(property);
        Self::new_raw(name, variant)
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is monotonic. Annotation is left empty for now.
    pub fn mk_fn_input_monotonic(
        name: &str,
        input_index: Option<usize>,
        target: Option<UninterpretedFnId>,
        value: Monotonicity,
    ) -> StatProperty {
        let property = FnInputMonotonic {
            input_index,
            target,
            value,
            context: None,
        };
        let variant = StatPropertyType::FnInputMonotonic(property);
        Self::new_raw(name, variant)
    }

    /// Create `StatProperty` instance describing that an input of an uninterpreted function
    /// is monotonic in a certain context. Annotation is left empty for now.
    pub fn mk_fn_input_monotonic_context(
        name: &str,
        input_index: Option<usize>,
        target: Option<UninterpretedFnId>,
        value: Monotonicity,
        context: String,
    ) -> StatProperty {
        let property = FnInputMonotonic {
            input_index,
            target,
            value,
            context: Some(context),
        };
        let variant = StatPropertyType::FnInputMonotonicContext(property);
        Self::new_raw(name, variant)
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
        Self::try_mk_generic("New generic static property", "true").unwrap()
    }

    /// Create default `StatProperty` instance for regulation essentiality (with empty `input` and
    /// `target` fields and `Unknown` essentiality).
    pub fn default_regulation_essential() -> StatProperty {
        Self::mk_regulation_essential(
            "Regulation essentiality (generated)",
            None,
            None,
            Essentiality::Unknown,
        )
    }

    /// Create default `StatProperty` instance for regulation essentiality in a context
    /// (with empty `input`, `target`, and `context` fields and `Unknown` essentiality).
    pub fn default_regulation_essential_context() -> StatProperty {
        Self::mk_regulation_essential_context(
            "New regulation essentiality property",
            None,
            None,
            Essentiality::Unknown,
            "true".to_string(),
        )
    }

    /// Create default `StatProperty` instance for regulation monotonicity (with empty `input` and
    /// `target` fields and `Unknown` monotonicity).
    pub fn default_regulation_monotonic() -> StatProperty {
        Self::mk_regulation_monotonic(
            "Regulation monotonicity (generated)",
            None,
            None,
            Monotonicity::Unknown,
        )
    }

    /// Create default `StatProperty` instance for regulation monotonicity in a context
    /// (with empty `input`, `target`, and `context` fields and `Unknown` monotonicity).
    pub fn default_regulation_monotonic_context() -> StatProperty {
        Self::mk_regulation_monotonic_context(
            "New regulation monotonicity property",
            None,
            None,
            Monotonicity::Unknown,
            "true".to_string(),
        )
    }

    /// Create default `StatProperty` instance for function input essentiality (with empty `input`
    /// and `target` fields and `Unknown` essentiality).
    pub fn default_fn_input_essential() -> StatProperty {
        Self::mk_fn_input_essential(
            "Fn input essentiality (generated)",
            None,
            None,
            Essentiality::Unknown,
        )
    }

    /// Create default `StatProperty` instance for function input essentiality in a context
    /// (with empty `input`, `target`, and `context` fields and `Unknown` essentiality).
    pub fn default_fn_input_essential_context() -> StatProperty {
        Self::mk_fn_input_essential_context(
            "New fn input essentiality property",
            None,
            None,
            Essentiality::Unknown,
            "true".to_string(),
        )
    }

    /// Create default `StatProperty` instance for function input monotonicity (with empty `input`
    /// and `target` fields and `Unknown` monotonicity).
    pub fn default_fn_input_monotonic() -> StatProperty {
        Self::mk_fn_input_monotonic(
            "Fn input monotonicity (generated)",
            None,
            None,
            Monotonicity::Unknown,
        )
    }

    /// Create default `StatProperty` instance for function input monotonicity in a context
    /// (with empty `input`, `target`, and `context` fields and `Unknown` monotonicity).
    pub fn default_fn_input_monotonic_context() -> StatProperty {
        Self::mk_fn_input_monotonic_context(
            "New fn input monotonicity property",
            None,
            None,
            Monotonicity::Unknown,
            "true".to_string(),
        )
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

    /// Set property's annotation string.
    pub fn set_annotation(&mut self, annotation: &str) {
        self.annotation = annotation.to_string()
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

    /// If the property is referencing the given variable (as either regulator or target),
    /// set that variable to the new value.
    ///
    /// This is applicable to all kinds of regulation properties.
    /// If not applicable, return `Err`.
    pub fn set_var_id_if_present(&mut self, old_id: VarId, new_id: VarId) -> Result<(), String> {
        let (reg_var, target_var) = self.get_regulator_and_target()?;
        if let Some(var_id) = reg_var {
            if var_id == old_id {
                self.set_input_var(new_id.clone())?;
            }
        }
        if let Some(var_id) = target_var {
            if var_id == old_id {
                self.set_target_var(new_id)?;
            }
        }
        Ok(())
    }

    /// If the property is referencing the given function, set its ID to the new value.
    ///
    /// This is applicable to all kinds of unintepreted fn properties.
    /// If not applicable, return `Err`.
    pub fn set_fn_id_if_present(
        &mut self,
        old_id: UninterpretedFnId,
        new_id: UninterpretedFnId,
    ) -> Result<(), String> {
        let (function, _) = self.get_function_and_index()?;
        if let Some(fn_id) = function {
            if fn_id == old_id {
                self.set_target_fn(new_id)?;
            }
        }
        Ok(())
    }
}

/// Observing static properties.
impl StatProperty {
    /// Get property's name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get annotation string.
    pub fn get_annotation(&self) -> &str {
        &self.annotation
    }

    /// Get property's variant with all the underlying data.
    pub fn get_prop_data(&self) -> &StatPropertyType {
        &self.variant
    }

    /// Check that the property has all required fields filled out.
    /// If some of the required field is set to None, return error.
    pub fn assert_fully_filled(&self) -> Result<(), String> {
        let missing_field_msg = "One of the required fields is not filled.";

        match &self.variant {
            StatPropertyType::GenericStatProp(_) => {} // no fields that can be None
            StatPropertyType::FnInputEssential(p)
            | StatPropertyType::FnInputEssentialContext(p) => {
                if p.input_index.is_none() || p.target.is_none() {
                    return Err(missing_field_msg.to_string());
                }
            }
            StatPropertyType::FnInputMonotonic(p)
            | StatPropertyType::FnInputMonotonicContext(p) => {
                if p.input_index.is_none() || p.target.is_none() {
                    return Err(missing_field_msg.to_string());
                }
            }
            StatPropertyType::RegulationEssential(p)
            | StatPropertyType::RegulationEssentialContext(p) => {
                if p.input.is_none() || p.target.is_none() {
                    return Err(missing_field_msg.to_string());
                }
            }
            StatPropertyType::RegulationMonotonic(p)
            | StatPropertyType::RegulationMonotonicContext(p) => {
                if p.input.is_none() || p.target.is_none() {
                    return Err(missing_field_msg.to_string());
                }
            }
        }
        Ok(())
    }

    /// If this is some kind of regulation property, get property's sub-fields for regulator
    /// variable and target variable. If not applicable (not a regulation property), return `Err`.
    ///
    /// This may be useful if we need to update all kinds of regulation properties.
    pub fn get_regulator_and_target(&mut self) -> Result<(Option<VarId>, Option<VarId>), String> {
        match &mut self.variant {
            StatPropertyType::RegulationMonotonic(prop) => {
                Ok((prop.input.clone(), prop.target.clone()))
            }
            StatPropertyType::RegulationMonotonicContext(prop) => {
                Ok((prop.input.clone(), prop.target.clone()))
            }
            StatPropertyType::RegulationEssential(prop) => {
                Ok((prop.input.clone(), prop.target.clone()))
            }
            StatPropertyType::RegulationEssentialContext(prop) => {
                Ok((prop.input.clone(), prop.target.clone()))
            }
            other_variant => Err(format!(
                "{other_variant:?} does not have fields for both regulator and target variable."
            )),
        }
    }

    /// If this is some kind of function property, get property's sub-fields for function ID
    /// and argument index. If not applicable (not a function property), return `Err`.
    ///
    /// This may be useful if we need to update all kinds of uninterpreted function properties.
    pub fn get_function_and_index(
        &mut self,
    ) -> Result<(Option<UninterpretedFnId>, Option<usize>), String> {
        match &mut self.variant {
            StatPropertyType::FnInputEssential(prop) => Ok((prop.target.clone(), prop.input_index)),
            StatPropertyType::FnInputEssentialContext(prop) => {
                Ok((prop.target.clone(), prop.input_index))
            }
            StatPropertyType::FnInputMonotonic(prop) => Ok((prop.target.clone(), prop.input_index)),
            StatPropertyType::FnInputMonotonicContext(prop) => {
                Ok((prop.target.clone(), prop.input_index))
            }
            other_variant => Err(format!(
                "{other_variant:?} does not have fields for function and argument index."
            )),
        }
    }
}

/// Static methods to create standard IDs for automatically generated static properties.
impl StatProperty {
    /// Get ID of a static property that describes monotonicity of a regulation
    /// between `regulator` and `target`.
    pub fn get_reg_monotonicity_prop_id(regulator: &VarId, target: &VarId) -> StatPropertyId {
        let id_str = format!("monotonicity_{regulator}_{target}");
        // this will always be a valid ID string, we can unwrap
        StatPropertyId::new(&id_str).unwrap()
    }

    /// Get ID of a static property that describes essentiality of a regulation
    /// between `regulator` and `target`.
    pub fn get_reg_essentiality_prop_id(regulator: &VarId, target: &VarId) -> StatPropertyId {
        let id_str = format!("essentiality_{regulator}_{target}");
        // this will always be a valid ID string, we can unwrap
        StatPropertyId::new(&id_str).unwrap()
    }

    /// Get ID of a static property that describes monotonicity of input on given `index`
    /// for function `fn_id`.
    pub fn get_fn_input_monotonicity_prop_id(
        fn_id: &UninterpretedFnId,
        index: usize,
    ) -> StatPropertyId {
        let id_str = format!("fn_monotonicity_{fn_id}_{index}");
        // this will always be a valid ID string, we can unwrap
        StatPropertyId::new(&id_str).unwrap()
    }

    /// Get ID of a static property that describes essentiality of input on given `index`
    /// for function `fn_id`.
    pub fn get_fn_input_essentiality_prop_id(
        fn_id: &UninterpretedFnId,
        index: usize,
    ) -> StatPropertyId {
        let id_str = format!("fn_essentiality_{fn_id}_{index}");
        // this will always be a valid ID string, we can unwrap
        StatPropertyId::new(&id_str).unwrap()
    }
}
