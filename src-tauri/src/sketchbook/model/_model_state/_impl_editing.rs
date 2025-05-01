use crate::sketchbook::data_structs::ModelData;
use crate::sketchbook::ids::{LayoutId, UninterpretedFnId, VarId};
use crate::sketchbook::layout::Layout;
use crate::sketchbook::model::{
    Essentiality, FnArgument, ModelState, Monotonicity, Regulation, UninterpretedFn, UpdateFn,
    Variable,
};
use crate::sketchbook::utils::assert_ids_unique;
use crate::sketchbook::Manager;
use std::collections::{HashMap, HashSet};

/// Methods for safely constructing or mutating instances of `ModelState`.
///
/// These methods focus on general manipulation with variables/regulations.
/// See below for API focusing on layout manipulation.
impl ModelState {
    /// Create a new `ModelState` that does not contain any `Variables`, `Uninterpreted Functions`,
    /// or `Regulations` yet. It contains a single empty default `Layout`.
    pub fn new_empty() -> ModelState {
        let default_layout_id = ModelState::get_default_layout_id();
        // `get_default_layout_name()` returns a valid name, so we can safely unwrap
        let default_layout = Layout::new_empty(ModelState::get_default_layout_name()).unwrap();
        ModelState {
            variables: HashMap::new(),
            regulations: HashSet::new(),
            update_fns: HashMap::new(),
            uninterpreted_fns: HashMap::new(),
            layouts: HashMap::from([(default_layout_id, default_layout)]),
        }
    }

    /// Create a new `ModelState` given a corresponding `ModelData` instance.
    pub fn new_from_model_data(model_data: &ModelData) -> Result<ModelState, String> {
        let mut model = ModelState::new_empty();
        // start with variables and plain function symbols (so that they can be used in expressions later)
        model_data
            .variables
            .iter()
            .try_for_each(|v| model.add_var_by_str(&v.id, &v.name, &v.annotation))?;
        model_data.uninterpreted_fns.iter().try_for_each(|f| {
            model.add_empty_uninterpreted_fn_by_str(&f.id, &f.name, f.arguments.len())?;
            model.set_fn_annot_by_str(&f.id, &f.annotation)
        })?;

        // add regulations
        model_data.regulations.iter().try_for_each(|r| {
            model.add_regulation(
                VarId::new(&r.regulator)?,
                VarId::new(&r.target)?,
                r.essential,
                r.sign,
            )
        })?;

        // add layouts
        for layout_data in &model_data.layouts {
            let layout = layout_data.to_layout()?;
            // check layout has valid variables
            model.add_or_update_layout_raw(LayoutId::new(&layout_data.id)?, layout)?;
        }

        // set update fns
        let update_fns = model_data
            .variables
            .iter()
            .map(|v| (v.id.as_str(), v.update_fn.as_str()))
            .collect();
        model.set_multiple_update_fns(update_fns)?;

        // set expressions, arguments, and annotations for uninterpreted fns
        for f in &model_data.uninterpreted_fns {
            model.set_uninterpreted_fn_expression_by_str(&f.id, &f.expression)?;
            model.set_fn_annot_by_str(&f.id, &f.annotation)?;
            let arguments = f
                .arguments
                .iter()
                .map(|(m, e)| FnArgument::new(*e, *m))
                .collect();
            model.set_uninterpreted_fn_all_args_by_str(&f.id, arguments)?;
        }
        Ok(model)
    }

    /// Create new `ModelState` using provided variable ID-name pairs, both strings.
    /// All variables have default (empty) update functions.
    /// Result will contain no `UninterpretedFns` or `Regulations`, and a single default `Layout`.
    ///
    /// The IDs must be unique valid identifiers.
    /// The names might be same as the IDs. It also might be empty or non-unique.
    /// The variable annotations are left empty.
    ///
    /// Return `Err` in case the IDs are not unique.
    pub fn new_from_vars(variables: Vec<(&str, &str)>) -> Result<ModelState, String> {
        let mut model = ModelState::new_empty();
        let var_ids = variables.iter().map(|pair| pair.0).collect();
        assert_ids_unique(&var_ids)?;

        for (id, var_name) in variables {
            let var_id = VarId::new(id)?;
            model
                .variables
                .insert(var_id.clone(), Variable::new(var_name)?);
            model.add_default_update_fn(var_id.clone())?;
            model.insert_to_default_layout(var_id)?;
        }
        Ok(model)
    }

    /// Add a new variable with given `var_id` and `name` to this `ModelState`. The variable
    /// will receive a default "empty" update function.
    ///
    /// The ID must be valid identifier that is not already used by some other variable.
    /// The names might be same as the ID. It also might be empty or non-unique.
    ///
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_var(&mut self, var_id: VarId, name: &str, annot: &str) -> Result<(), String> {
        self.assert_no_variable(&var_id)?;
        let variable = Variable::new_annotated(name, annot)?;
        self.variables.insert(var_id.clone(), variable);
        self.add_default_update_fn(var_id.clone())?;
        self.insert_to_all_layouts(var_id)?;
        Ok(())
    }

    /// Add a new variable with given `id` and `name` to this `ModelState`. The variable
    /// will receive a default "empty" update function.
    ///
    /// The ID must be valid identifier that is not already used by some other variable.
    /// The names might be same as the ID. It also might be empty or non-unique.
    ///
    /// Returns `Err` in case the `id` is not a valid identifier or if it is already being used.
    pub fn add_var_by_str(&mut self, id: &str, name: &str, annot: &str) -> Result<(), String> {
        let var_id = VarId::new(id)?;
        self.add_var(var_id, name, annot)
    }

    /// Shorthand to add a list of new variables with given string IDs and names to this `ModelState`.
    ///
    /// Each ID must be valid identifier that is not already used by some other variable.
    /// The names might be same as the ID. It also might be empty or non-unique.
    /// Variable annotations are left empty.
    ///
    /// Returns `Err` in case some `id` is already being used.
    pub fn add_multiple_variables(
        &mut self,
        id_name_pairs: Vec<(&str, &str)>,
    ) -> Result<(), String> {
        // before making any changes, check that all IDs are actually valid and unique
        let var_ids: Vec<&str> = id_name_pairs.iter().map(|pair| pair.0).collect();
        self.assert_ids_unique_and_new(&var_ids, &(Self::assert_no_variable))?;
        for id in var_ids {
            let var_id = VarId::new(id)?;
            self.assert_no_variable(&var_id)?;
        }
        // now we can safely add them
        for (id, name) in id_name_pairs {
            self.add_var_by_str(id, name, "")?;
        }
        Ok(())
    }

    /// Add a new uninterpreted fn given by its components.
    pub fn add_uninterpreted_fn(
        &mut self,
        fn_id: UninterpretedFnId,
        name: &str,
        arguments: Vec<FnArgument>,
        expression: &str,
        annot: &str,
    ) -> Result<(), String> {
        self.assert_no_uninterpreted_fn(&fn_id)?;
        let f = UninterpretedFn::new(name, annot, expression, arguments, self, &fn_id)?;
        self.uninterpreted_fns.insert(fn_id, f);
        Ok(())
    }

    /// Add a new uninterpreted fn with given `id`, `name` and `arity` to this `ModelState`.
    /// Note that constraints regarding monotonicity or essentiality must be added separately.
    ///
    /// The ID must be valid identifier that is not already used by some other uninterpreted fn.
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_empty_uninterpreted_fn(
        &mut self,
        fn_id: UninterpretedFnId,
        name: &str,
        arity: usize,
    ) -> Result<(), String> {
        self.assert_no_uninterpreted_fn(&fn_id)?;
        self.uninterpreted_fns.insert(
            fn_id,
            UninterpretedFn::new_without_constraints(name, arity)?,
        );
        Ok(())
    }

    /// Add a new uninterpreted fn with given string `id`, `name`, and `arity` to this `ModelState`.
    ///
    /// The ID must be valid identifier that is not already used by some other uninterpreted fn.
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_empty_uninterpreted_fn_by_str(
        &mut self,
        id: &str,
        name: &str,
        arity: usize,
    ) -> Result<(), String> {
        let fn_id = UninterpretedFnId::new(id)?;
        self.add_empty_uninterpreted_fn(fn_id, name, arity)
    }

    /// Shorthand to add a list of new uninterpreted fns, each with a string ID, name, and arity,
    /// to this `ModelState`. Details (incl. annotations) for these functions are left empty.
    ///
    /// Each ID must be valid identifier that is not already used by some other uninterpreted fns.
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_multiple_uninterpreted_fns(
        &mut self,
        id_name_arity_tuples: Vec<(&str, &str, usize)>,
    ) -> Result<(), String> {
        // before making any changes, check that all IDs are actually valid and unique
        let fn_ids = id_name_arity_tuples
            .iter()
            .map(|triplet| triplet.0)
            .collect();
        self.assert_ids_unique_and_new(&fn_ids, &(Self::assert_no_uninterpreted_fn))?;
        // now we can safely add them
        for (id, name, arity) in id_name_arity_tuples {
            self.add_empty_uninterpreted_fn_by_str(id, name, arity)?;
        }
        Ok(())
    }

    /// Add a new `Regulation` to this `ModelState`.
    ///
    /// Returns `Err` when one of the variables is invalid, or the regulation between the two
    /// variables already exists.
    pub fn add_regulation(
        &mut self,
        regulator: VarId,
        target: VarId,
        essential: Essentiality,
        regulation_sign: Monotonicity,
    ) -> Result<(), String> {
        self.assert_valid_variable(&regulator)?;
        self.assert_valid_variable(&target)?;
        self.assert_no_regulation(&regulator, &target)?;
        self.regulations.insert(Regulation::new(
            regulator,
            target,
            essential,
            regulation_sign,
        ));
        Ok(())
    }

    /// Add a new `Regulation` to this `ModelState` using a string representation. The
    /// variables in the given string must be valid ID strings for this `ModelState`.
    ///
    /// Returns `Err` when the string does not encode a valid regulation, if the provided variables
    /// are not valid variable IDs, or when the regulation between the two variables already exists.
    pub fn add_regulation_by_str(&mut self, regulation_str: &str) -> Result<(), String> {
        let (reg, regulation_sign, essential, tar) =
            Regulation::try_components_from_string(regulation_str)?;
        let regulator = VarId::new(reg.as_str())?;
        let target = VarId::new(tar.as_str())?;
        // all validity checks inside
        self.add_regulation(regulator, target, essential, regulation_sign)
    }

    /// Shorthand to add a list of new `Regulations` given by their string encoding to this `ModelState`.
    /// The variables in the given string must be valid ID strings for this `ModelState`.
    ///
    /// Returns `Err` when the string does not encode a valid regulation, if the provided variables
    /// are not valid variable IDs, or when the regulation between the two variables already exists.
    pub fn add_multiple_regulations(&mut self, regulations: Vec<&str>) -> Result<(), String> {
        // before making any changes, check that all regulations are actually valid
        for regulation_str in regulations.iter() {
            let (reg, _, _, tar) = Regulation::try_components_from_string(regulation_str)?;
            let regulator = VarId::new(reg.as_str())?;
            let target = VarId::new(tar.as_str())?;
            self.assert_no_regulation(&regulator, &target)?
        }

        for regulation_str in regulations {
            self.add_regulation_by_str(regulation_str)?;
        }
        Ok(())
    }

    /// Set the raw variable data for a variable `var_id`.
    pub fn set_raw_var(&mut self, var_id: &VarId, var_data: Variable) -> Result<(), String> {
        self.assert_valid_variable(var_id)?;
        self.variables.insert(var_id.clone(), var_data);
        Ok(())
    }

    /// Set the raw uninterpreted function data for a function `fn_id`.
    pub fn set_raw_function(
        &mut self,
        fn_id: &UninterpretedFnId,
        fn_data: UninterpretedFn,
    ) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(fn_id)?;
        self.uninterpreted_fns.insert(fn_id.clone(), fn_data);
        Ok(())
    }

    /// Set the name of a network variable given by id `var_id`.
    /// The name does not have to be unique, as multiple variables might share a name.
    ///
    /// Note that you don't have to rename anything else in the network, since all other
    /// structures reference variables with ids.
    pub fn set_var_name(&mut self, var_id: &VarId, name: &str) -> Result<(), String> {
        self.assert_valid_variable(var_id)?;
        let variable = self.variables.get_mut(var_id).unwrap();
        variable.set_name(name)?;
        Ok(())
    }

    /// Set the name of a network variable given by string `id`.
    ///
    /// The name does not have to be unique, as multiple variables might share a name.
    pub fn set_var_name_by_str(&mut self, id: &str, name: &str) -> Result<(), String> {
        let var_id = VarId::new(id)?;
        self.set_var_name(&var_id, name)
    }

    /// Set the annotation of a network variable given by id `var_id`.
    pub fn set_var_annot(&mut self, var_id: &VarId, annot: &str) -> Result<(), String> {
        self.assert_valid_variable(var_id)?;
        let variable = self.variables.get_mut(var_id).unwrap();
        variable.set_annotation(annot);
        Ok(())
    }

    /// Set the annotation of a network variable given by id `var_id`.
    pub fn set_var_annot_by_str(&mut self, id: &str, annot: &str) -> Result<(), String> {
        let var_id = VarId::new(id)?;
        self.set_var_annot(&var_id, annot)
    }

    /// Set the id of variable with `original_id` to `new_id`.
    ///
    /// Note that this operation may be costly as it affects several components of the state.
    pub fn set_var_id(&mut self, original_id: &VarId, new_id: VarId) -> Result<(), String> {
        self.assert_valid_variable(original_id)?;
        self.assert_no_variable(&new_id)?;

        // all changes must be done directly, not through some helper fns, because the state
        // might not be consistent inbetween various deletions

        // 1) change id in variables list
        if let Some(variable) = self.variables.remove(original_id) {
            self.variables.insert(new_id.clone(), variable);
        }

        // 2) change id in each regulation
        let regs_to_update: Vec<Regulation> = self
            .regulations
            .iter()
            .filter(|reg| reg.get_regulator() == original_id || reg.get_target() == original_id)
            .cloned()
            .collect();
        for reg in regs_to_update {
            self.regulations.remove(&reg);
            let mut updated_reg = reg.clone();
            if reg.get_regulator() == original_id {
                updated_reg.swap_regulator(new_id.clone());
            }
            if reg.get_target() == original_id {
                updated_reg.swap_target(new_id.clone());
            }
            self.regulations.insert(updated_reg);
        }

        // 3) change var id in each layout
        for layout in self.layouts.values_mut() {
            layout.change_node_id(original_id, new_id.clone())?;
        }

        // 4) change key in update functions hashmap
        if let Some(update_fn) = self.update_fns.remove(original_id) {
            self.update_fns.insert(new_id.clone(), update_fn);
        }

        // 5) substitute id for this variable in all update functions
        for var_id in self.variables.keys() {
            let update_fn = self.update_fns.remove(var_id).unwrap();
            let new_update_fn =
                UpdateFn::with_changed_var_id(update_fn, original_id, &new_id, self);
            self.update_fns.insert(var_id.clone(), new_update_fn);
        }

        Ok(())
    }

    /// Set the id of variable given by string `original_id` to `new_id`.
    ///
    /// Note that this operation may be costly as it affects several components.
    pub fn set_var_id_by_str(&mut self, original_id: &str, new_id: &str) -> Result<(), String> {
        let original_id = VarId::new(original_id)?;
        let new_id = VarId::new(new_id)?;
        self.set_var_id(&original_id, new_id)
    }

    /// Remove the network variable with given `var_id` from this `ModelState`. This also
    /// removes the variable from all `Layouts`, removes its `UpdateFn` and all `Regulations`
    /// where this variable figures.
    ///
    /// Returns `Err` in case the `var_id` is not a valid variable's identifier, or if some update function
    /// depends on it.
    pub fn remove_var(&mut self, var_id: &VarId) -> Result<(), String> {
        self.assert_valid_variable(var_id)?;

        // check that variable can be safely deleted (not contained in any update fn)
        if self.is_var_contained_in_updates(var_id) {
            return Err(format!(
                "Cannot remove variable `{var_id}`, it is still contained in an update function."
            ));
        }

        // first delete all regulations, layout nodes, and lastly the variable itself
        self.remove_all_regulations_var(var_id)?;
        self.remove_from_all_layouts(var_id)?;

        if self.update_fns.remove(var_id).is_none() {
            panic!("Error when removing update fn for variable {var_id}.")
        }
        if self.variables.remove(var_id).is_none() {
            panic!("Error when removing variable {var_id} from the variable map.")
        }
        Ok(())
    }

    /// Remove the network variable with given `var_id` from this `ModelState`. This also
    /// removes the variable from all `Layouts`, removes its `UpdateFn` and all `Regulations`
    /// where this variable figures.
    ///
    /// Returns `Err` in case the `var_id` is not a valid variable's identifier.
    pub fn remove_var_by_str(&mut self, id: &str) -> Result<(), String> {
        let var_id = VarId::new(id)?;
        self.remove_var(&var_id)
    }

    /// Set the name of an uninterpreted fn given by id `fn_id`.
    pub fn set_uninterpreted_fn_name(
        &mut self,
        fn_id: &UninterpretedFnId,
        name: &str,
    ) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(fn_id)?;
        let uninterpreted_fn = self.uninterpreted_fns.get_mut(fn_id).unwrap();
        uninterpreted_fn.set_name(name)?;
        Ok(())
    }

    /// Set the name of an uninterpreted fn given by string `id`.
    pub fn set_uninterpreted_fn_name_by_str(&mut self, id: &str, name: &str) -> Result<(), String> {
        let fn_id = UninterpretedFnId::new(id)?;
        self.set_uninterpreted_fn_name(&fn_id, name)
    }

    /// Set annotation of an uninterpreted fn given by id `fn_id`.
    pub fn set_fn_annot(&mut self, fn_id: &UninterpretedFnId, annot: &str) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(fn_id)?;
        let uninterpreted_fn = self.uninterpreted_fns.get_mut(fn_id).unwrap();
        uninterpreted_fn.set_annotation(annot);
        Ok(())
    }

    /// Set annotation of an uninterpreted fn given by string `id`.
    pub fn set_fn_annot_by_str(&mut self, id: &str, annot: &str) -> Result<(), String> {
        let fn_id = UninterpretedFnId::new(id)?;
        self.set_fn_annot(&fn_id, annot)
    }

    /// Set arity of an uninterpreted fn given by id `fn_id`.
    ///
    /// In order to change arity of a function symbol, it must not currently be used in any
    /// update/uninterpreted function's expression (because in expressions, it is applied on a
    /// fixed number of arguments).
    pub fn set_uninterpreted_fn_arity(
        &mut self,
        fn_id: &UninterpretedFnId,
        arity: usize,
    ) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(fn_id)?;
        self.assert_fn_not_used_in_expressions(fn_id)?;

        let uninterpreted_fn = self.uninterpreted_fns.get_mut(fn_id).unwrap();
        uninterpreted_fn.set_arity(arity)?;
        Ok(())
    }

    /// Set the arity of an uninterpreted fn given by string `id`.
    pub fn set_uninterpreted_fn_arity_by_str(
        &mut self,
        id: &str,
        arity: usize,
    ) -> Result<(), String> {
        let fn_id = UninterpretedFnId::new(id)?;
        self.set_uninterpreted_fn_arity(&fn_id, arity)
    }

    /// Set expression of an uninterpreted fn given by id `fn_id`.
    pub fn set_uninterpreted_fn_expression(
        &mut self,
        fn_id: &UninterpretedFnId,
        expression: &str,
    ) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(fn_id)?;
        let original_fn = self.uninterpreted_fns.get(fn_id).unwrap().clone();
        // this will correctly return error if the expression is invalid
        let updated_fn =
            UninterpretedFn::with_new_expression(original_fn, expression, self, fn_id)?;
        self.uninterpreted_fns.insert(fn_id.clone(), updated_fn);
        Ok(())
    }

    /// Set expression of an uninterpreted fn given by string `id`.
    pub fn set_uninterpreted_fn_expression_by_str(
        &mut self,
        id: &str,
        expression: &str,
    ) -> Result<(), String> {
        let fn_id = UninterpretedFnId::new(id)?;
        self.set_uninterpreted_fn_expression(&fn_id, expression)
    }

    /// Set essentiality of an argument of given uninterpreted fn (on provided index).
    pub fn set_uninterpreted_fn_essentiality(
        &mut self,
        fn_id: &UninterpretedFnId,
        essentiality: Essentiality,
        index: usize,
    ) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(fn_id)?;
        let uninterpreted_fn = self.uninterpreted_fns.get_mut(fn_id).unwrap();
        uninterpreted_fn.set_essential(index, essentiality)?;
        Ok(())
    }

    /// Set essentiality of an argument of given uninterpreted fn (on provided index).
    pub fn set_uninterpreted_fn_essentiality_by_str(
        &mut self,
        id: &str,
        essentiality: Essentiality,
        index: usize,
    ) -> Result<(), String> {
        let fn_id = UninterpretedFnId::new(id)?;
        self.set_uninterpreted_fn_essentiality(&fn_id, essentiality, index)
    }

    /// Set monotonicity of an argument of given uninterpreted fn (on provided index).
    pub fn set_uninterpreted_fn_monotonicity(
        &mut self,
        fn_id: &UninterpretedFnId,
        monotonicity: Monotonicity,
        index: usize,
    ) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(fn_id)?;
        let uninterpreted_fn = self.uninterpreted_fns.get_mut(fn_id).unwrap();
        uninterpreted_fn.set_monotonic(index, monotonicity)?;
        Ok(())
    }

    /// Set monotonicity of an argument of given uninterpreted fn (on provided index).
    pub fn set_uninterpreted_fn_monotonicity_by_str(
        &mut self,
        id: &str,
        monotonicity: Monotonicity,
        index: usize,
    ) -> Result<(), String> {
        let fn_id = UninterpretedFnId::new(id)?;
        self.set_uninterpreted_fn_monotonicity(&fn_id, monotonicity, index)
    }

    /// Set constraints on all arguments of given uninterpreted fn.
    pub fn set_uninterpreted_fn_all_args(
        &mut self,
        fn_id: &UninterpretedFnId,
        fn_arguments: Vec<FnArgument>,
    ) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(fn_id)?;
        let uninterpreted_fn = self.uninterpreted_fns.get_mut(fn_id).unwrap();
        uninterpreted_fn.set_all_arguments(fn_arguments)?;
        Ok(())
    }

    /// Set constraints on all arguments of given uninterpreted fn.
    pub fn set_uninterpreted_fn_all_args_by_str(
        &mut self,
        id: &str,
        fn_arguments: Vec<FnArgument>,
    ) -> Result<(), String> {
        let fn_id = UninterpretedFnId::new(id)?;
        self.set_uninterpreted_fn_all_args(&fn_id, fn_arguments)
    }

    /// Set the id of an uninterpreted fn with `original_id` to `new_id`.
    ///
    /// Note that this operation may be costly as it affects several components of the state.
    pub fn set_uninterpreted_fn_id(
        &mut self,
        original_id: &UninterpretedFnId,
        new_id: UninterpretedFnId,
    ) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(original_id)?;
        self.assert_no_uninterpreted_fn(&new_id)?;

        // all changes must be done directly, not through some helper fns, because the state
        // might not be consistent in between various deletions

        // change key in uninterpreted functions hashmap
        if let Some(uninterpreted_fn) = self.uninterpreted_fns.remove(original_id) {
            self.uninterpreted_fns
                .insert(new_id.clone(), uninterpreted_fn);
        }

        // substitute id for this uninterpreted fn in all uninterpreted functions' expressions
        // TODO: there may be more efficient way to do this
        for fn_id in self.uninterpreted_fns.clone().keys() {
            let uninterpreted_fn = self.uninterpreted_fns.remove(fn_id).unwrap();
            let new_uninterpreted_fn =
                UninterpretedFn::with_changed_fn_id(uninterpreted_fn, original_id, &new_id, self);
            self.uninterpreted_fns
                .insert(fn_id.clone(), new_uninterpreted_fn);
        }

        // substitute id for this uninterpreted fn in all update functions
        for var_id in self.variables.keys() {
            let update_fn = self.update_fns.remove(var_id).unwrap();
            let new_update_fn = UpdateFn::with_changed_fn_id(update_fn, original_id, &new_id, self);
            self.update_fns.insert(var_id.clone(), new_update_fn);
        }

        Ok(())
    }

    /// Set the id of an uninterpreted fn given by string `original_id` to `new_id`.
    pub fn set_uninterpreted_fn_id_by_str(
        &mut self,
        original_id: &str,
        new_id: &str,
    ) -> Result<(), String> {
        let original_id = UninterpretedFnId::new(original_id)?;
        let new_id = UninterpretedFnId::new(new_id)?;
        self.set_uninterpreted_fn_id(&original_id, new_id)
    }

    /// Remove the uninterpreted fn with given `fn_id` from this `ModelState`. Note that this
    /// uninterpreted fn must not be used in any update fn.
    ///
    /// Also returns `Err` in case the `fn_id` is not a valid uninterpreted fn's identifier or if some
    /// update/uninterpreted function depends on it.
    pub fn remove_uninterpreted_fn(&mut self, fn_id: &UninterpretedFnId) -> Result<(), String> {
        self.assert_valid_uninterpreted_fn(fn_id)?;

        // check that this function symbol can be safely deleted (not contained in any update/uninterpreted fn)
        self.assert_fn_not_used_in_expressions(fn_id)?;

        if self.uninterpreted_fns.remove(fn_id).is_none() {
            panic!("Error when removing uninterpreted fn {fn_id} from the uninterpreted_fn map.")
        }
        Ok(())
    }

    /// Remove the uninterpreted_fn with given string `id` from this `ModelState`. Note that this
    /// uninterpreted_fn must not be used in any update fn.
    ///
    /// Also returns `Err` in case the `fn_id` is not a valid uninterpreted_fn's identifier.
    pub fn remove_uninterpreted_fn_by_str(&mut self, id: &str) -> Result<(), String> {
        let fn_id = UninterpretedFnId::new(id)?;
        self.remove_uninterpreted_fn(&fn_id)
    }

    /// Remove a `Regulation` pointing from `regulator` to `target` from this `ModelState`.
    ///
    /// Returns `Err` when one of the variables is invalid, or the regulation between the two
    /// variables does not exist.
    pub fn remove_regulation(&mut self, regulator: &VarId, target: &VarId) -> Result<(), String> {
        // all validity checks are performed inside
        let regulation = self.get_regulation(regulator, target)?.clone();
        if !self.regulations.remove(&regulation) {
            panic!("Error when removing regulation between {regulator} - {target}.")
        }
        Ok(())
    }

    /// Remove a `Regulation` given by `regulation_str` from this `ModelState`. The
    /// variables in the `regulation_str` must be valid ID strings for this `ModelState`.
    ///
    /// Returns `Err` when one of the variables is invalid, or the regulation between the two
    /// variables does not exist.
    pub fn remove_regulation_by_str(&mut self, regulation_str: &str) -> Result<(), String> {
        let regulation = Regulation::try_from_string(regulation_str)?;
        self.remove_regulation(regulation.get_regulator(), regulation.get_target())
    }

    /// Shorthand to change sign of a `Regulation` pointing from `regulator` to `target`.
    /// Currently it basically removes the regulation, and adds a new one with the new sign.
    ///
    /// Returns `Err` when one of the variables is invalid
    pub fn change_regulation_sign(
        &mut self,
        regulator: &VarId,
        target: &VarId,
        new_sign: &Monotonicity,
    ) -> Result<(), String> {
        // all validity checks are performed inside
        let regulation = self.get_regulation(regulator, target)?.clone();
        self.remove_regulation(regulation.get_regulator(), regulation.get_target())?;
        self.add_regulation(
            regulator.clone(),
            target.clone(),
            *regulation.get_essentiality(),
            *new_sign,
        )?;
        Ok(())
    }

    /// Shorthand to change essentiality of a `Regulation` pointing from `regulator` to `target`.
    /// Currently it basically removes the regulation, and adds a new one with the new essentiality value.
    ///
    /// Returns `Err` when one of the variables is invalid
    pub fn change_regulation_essentiality(
        &mut self,
        regulator: &VarId,
        target: &VarId,
        new_essentiality: &Essentiality,
    ) -> Result<(), String> {
        // all validity checks are performed inside
        let regulation = self.get_regulation(regulator, target)?.clone();
        self.remove_regulation(regulation.get_regulator(), regulation.get_target())?;
        self.add_regulation(
            regulator.clone(),
            target.clone(),
            *new_essentiality,
            *regulation.get_sign(),
        )?;
        Ok(())
    }

    /// Set update function for a given variable to a provided expression.
    pub fn set_update_fn(&mut self, var_id: &VarId, expression: &str) -> Result<(), String> {
        self.assert_valid_variable(var_id)?;

        // this will correctly return error if the expression is invalid
        let new_update_fn = UpdateFn::try_from_str(expression, self)?;
        self.update_fns.insert(var_id.clone(), new_update_fn);
        Ok(())
    }

    /// Set update functions for multiple variables (given ID-function pairs).
    /// The IDs must be unique valid identifiers.
    pub fn set_multiple_update_fns(
        &mut self,
        update_functions: Vec<(&str, &str)>,
    ) -> Result<(), String> {
        // before making any changes, we must perform all validity checks
        // -> check IDs are unique, correspond to existing variables, and that expressions are valid
        let var_ids = update_functions.iter().map(|pair| pair.0).collect();
        self.assert_ids_unique_and_used(&var_ids, &(Self::assert_valid_variable))?;
        let parsed_fns = update_functions
            .iter()
            .map(|(_, expression)| UpdateFn::try_from_str(expression, self))
            .collect::<Result<Vec<UpdateFn>, String>>()?;

        // now we can just simply add them all
        for (i, parsed_fn) in parsed_fns.into_iter().enumerate() {
            self.update_fns.insert(VarId::new(var_ids[i])?, parsed_fn);
        }
        Ok(())
    }

    /// **(internal)** Utility method to add a default update fn for a given variable.
    fn add_default_update_fn(&mut self, var_id: VarId) -> Result<(), String> {
        self.assert_valid_variable(&var_id)?;
        self.update_fns.insert(var_id, UpdateFn::default());
        Ok(())
    }

    /// **(internal)** Remove all `Regulations` where `variable` figures (as either regulator or
    /// target) from this `ModelState`.
    /// Returns `Err` when the variable is invalid.
    fn remove_all_regulations_var(&mut self, variable: &VarId) -> Result<(), String> {
        self.assert_valid_variable(variable)?;
        self.regulations
            .retain(|r| r.get_regulator() != variable && r.get_target() != variable);
        Ok(())
    }
}

/// Several utility methods to manipulate with layouts.
impl ModelState {
    /// Add a new `Layout` with given `layout_id` and `name` to this `ModelState`. The layout
    /// will contain nodes for all model's variables, all of them located at a default position.
    ///
    /// Returns `Err` if `layout_id` is already being used for some other `Layout`.
    pub fn add_layout_simple(&mut self, layout_id: LayoutId, name: &str) -> Result<(), String> {
        self.assert_no_layout(&layout_id)?;
        let variable_ids = self.variables.clone().into_keys().collect();
        let layout = Layout::new_from_vars_default(name, variable_ids)?;
        self.layouts.insert(layout_id, layout);
        Ok(())
    }

    /// Add a new (pre-generated) `Layout` with given `id` to this `ModelState`, or update
    /// existing if the `id` is already used. The layout must contain nodes for exactly all model's
    /// variables.
    pub fn add_or_update_layout_raw(&mut self, id: LayoutId, layout: Layout) -> Result<(), String> {
        let model_vars: HashSet<_> = self.variables.keys().collect();
        let layout_vars: HashSet<_> = layout.layout_nodes().map(|(v, _)| v).collect();
        if model_vars != layout_vars {
            return Err("Model variables and layout variables are different.".to_string());
        }

        self.layouts.insert(id, layout);
        Ok(())
    }

    /// Add a new `Layout` with given `layout_id` and `name` to this `ModelState`. The layout
    /// will be a direct copy of another existing layout given by id `template_layout_id`.
    ///
    /// Returns `Err` if `layout_id` is already being used for some other `Layout` in
    /// this `ModelState`, or if `template_layout_id` does not exist.
    pub fn add_layout_copy(
        &mut self,
        layout_id: LayoutId,
        name: &str,
        template_layout_id: &LayoutId,
    ) -> Result<(), String> {
        self.assert_no_layout(&layout_id)?;
        self.assert_valid_layout(template_layout_id)?;

        let template_layout = self.get_layout(template_layout_id)?;
        let layout = Layout::new_from_another_copy(name, template_layout);
        self.layouts.insert(layout_id, layout);
        Ok(())
    }

    /// Remove a `Layout` with given `layout_id` from this `ModelState`. Default layout
    /// can not be deleted.
    ///
    /// Returns `Err` in case the `id` is not a valid identifier in this `ModelState`.
    pub fn remove_layout(&mut self, layout_id: &LayoutId) -> Result<(), String> {
        self.assert_valid_layout(layout_id)?;
        if *layout_id == ModelState::get_default_layout_id() {
            return Err("Default layout can not be deleted.".to_string());
        }
        if self.layouts.remove(layout_id).is_none() {
            panic!("Error when removing layout {layout_id} from the layout map.")
        }
        Ok(())
    }

    /// Remove a `Layout` with given `id` from this `ModelState`.
    ///
    /// Returns `Err` in case the `id` is not a valid identifier in this `ModelState`.
    pub fn remove_layout_by_str(&mut self, id: &str) -> Result<(), String> {
        let layout_id = LayoutId::new(id)?;
        self.remove_layout(&layout_id)
    }

    /// Update position of a node for variable `var_id` in layout `layout_id`.
    ///
    /// Returns `Err` in case one of the ids is not a valid for this `ModelState`.
    pub fn update_position(
        &mut self,
        layout_id: &LayoutId,
        var_id: &VarId,
        px: f32,
        py: f32,
    ) -> Result<(), String> {
        self.assert_valid_layout(layout_id)?;
        self.assert_valid_variable(var_id)?;

        self.layouts
            .get_mut(layout_id)
            .ok_or(format!("Error accessing layout {layout_id} in layout map"))?
            .update_node_position(var_id, px, py)
    }

    /// **(internal)** Utility method to add a variable node to a given layout.
    /// The node is inserted to a default position x=0,y=0.
    fn insert_to_layout(&mut self, var_id: VarId, layout_id: &LayoutId) -> Result<(), String> {
        self.assert_valid_variable(&var_id)?;
        self.assert_valid_layout(layout_id)?;

        let layout = self.layouts.get_mut(layout_id).unwrap();
        layout.add_default_node(var_id)?;
        Ok(())
    }

    /// **(internal)** Shorthand method for adding a variable node to a default layout.
    /// The node is inserted to a default position x=0,y=0.
    fn insert_to_default_layout(&mut self, var_id: VarId) -> Result<(), String> {
        let default_layout_id = ModelState::get_default_layout_id();
        self.insert_to_layout(var_id, &default_layout_id)
    }

    /// **(internal)** Shorthand method for adding a variable node to all layouts.
    /// The node is always inserted to a default position x=0,y=0.
    fn insert_to_all_layouts(&mut self, var_id: VarId) -> Result<(), String> {
        for layout in self.layouts.values_mut() {
            layout.add_default_node(var_id.clone())?;
        }
        Ok(())
    }

    /// **(internal)** Utility method to remove a variable node from all layouts.
    fn remove_from_all_layouts(&mut self, var_id: &VarId) -> Result<(), String> {
        for layout in self.layouts.values_mut() {
            layout.remove_node(var_id)?
        }
        Ok(())
    }
}

/// Several utility methods to assert (non-)existence of variables/regulations/layouts in the
/// current state.
impl ModelState {
    /// **(internal)** Utility method to ensure there is no regulation between the two variables yet.
    fn assert_no_regulation(&self, regulator: &VarId, target: &VarId) -> Result<(), String> {
        if self.get_regulation(regulator, target).is_err() {
            Ok(())
        } else {
            Err(format!(
                "Invalid regulation: {regulator} already regulates {target}."
            ))
        }
    }

    /// **(internal)** Utility method to ensure there is no variable with given Id yet.
    fn assert_no_variable(&self, var_id: &VarId) -> Result<(), String> {
        if self.is_valid_var_id(var_id) {
            Err(format!(
                "Variable with id {var_id} already exists in this model."
            ))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a variable with given Id.
    fn assert_valid_variable(&self, var_id: &VarId) -> Result<(), String> {
        if self.is_valid_var_id(var_id) {
            Ok(())
        } else {
            Err(format!(
                "Variable with id {var_id} does not exist in this model."
            ))
        }
    }

    /// **(internal)** Utility method to ensure there is no uninterpreted_fn with given Id yet.
    fn assert_no_uninterpreted_fn(&self, fn_id: &UninterpretedFnId) -> Result<(), String> {
        if self.is_valid_uninterpreted_fn_id(fn_id) {
            Err(format!("UninterpretedFn with id {fn_id} already exists."))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a uninterpreted fn with given Id.
    fn assert_valid_uninterpreted_fn(&self, fn_id: &UninterpretedFnId) -> Result<(), String> {
        if self.is_valid_uninterpreted_fn_id(fn_id) {
            Ok(())
        } else {
            Err(format!("UninterpretedFn with id {fn_id} does not exist."))
        }
    }

    /// **(internal)** Utility method to ensure there is no layout with given Id yet.
    fn assert_no_layout(&self, layout_id: &LayoutId) -> Result<(), String> {
        if self.is_valid_layout_id(layout_id) {
            Err(format!("Layout with id {layout_id} already exists."))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a layout with given Id.
    fn assert_valid_layout(&self, layout_id: &LayoutId) -> Result<(), String> {
        if self.is_valid_layout_id(layout_id) {
            Ok(())
        } else {
            Err(format!("Layout with id {layout_id} does not exist."))
        }
    }

    /// **(internal)** Utility method to ensure that an uninterpreted function is not used in any
    /// expressions (corresponding to any update function or any uninterpreted function).
    fn assert_fn_not_used_in_expressions(&self, fn_id: &UninterpretedFnId) -> Result<(), String> {
        // check that this function symbol can be safely deleted (not contained in any update/uninterpreted fn)
        let mut fn_symbols = HashSet::new();
        for update_fn in self.update_fns.values() {
            let tmp_fn_symbols = update_fn.collect_fn_symbols();
            fn_symbols.extend(tmp_fn_symbols);
        }
        for uninterpreted_fn in self.uninterpreted_fns.values() {
            let tmp_fn_symbols = uninterpreted_fn.collect_fn_symbols();
            fn_symbols.extend(tmp_fn_symbols);
        }
        if fn_symbols.contains(fn_id) {
            Err(format!("Cannot alter fn symbol `{fn_id}`, it is currently used in some update/uninterpreted function."))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::layout::NodePosition;
    use crate::sketchbook::model::{Essentiality, ModelState, Monotonicity};

    /// Test generating new default variant of the `ModelState`.
    #[test]
    fn test_new_default() {
        let model = ModelState::new_empty();
        assert_eq!(model.num_vars(), 0);
        assert_eq!(model.num_uninterpreted_fns(), 0);
        assert_eq!(model.highest_uninterpreted_fn_arity(), 0);
        assert_eq!(model.num_regulations(), 0);
        assert_eq!(model.num_layouts(), 1);

        let default_layout_id = ModelState::get_default_layout_id();
        assert_eq!(
            model.get_layout_name(&default_layout_id).unwrap().as_str(),
            ModelState::get_default_layout_name(),
        )
    }

    /// Test generating new `ModelState` from a set of variables.
    #[test]
    fn test_new_from_vars() {
        let var_id_name_pairs = vec![("a_id", "a_name"), ("a_id", "b_name")];
        assert!(ModelState::new_from_vars(var_id_name_pairs).is_err());

        let var_id_name_pairs = vec![("a_id", "a_name"), ("b_id", "b_name")];
        let model = ModelState::new_from_vars(var_id_name_pairs).unwrap();
        assert_eq!(model.num_vars(), 2);
        assert!(model.is_valid_var_id_str("a_id"));
        assert!(model.is_valid_var_id_str("b_id"));
        assert!(!model.is_valid_var_id_str("c_id"));
    }

    /// Test adding variables (both incrementally and at once).
    #[test]
    fn test_adding_variables() {
        let mut model = ModelState::new_empty();

        // one by one add variables a, b, c
        model.add_var_by_str("a", "a_name", "").unwrap();
        model.add_var_by_str("b", "b_name", "").unwrap();
        model.add_var_by_str("c", "c_name", "").unwrap();
        assert_eq!(model.num_vars(), 3);
        assert!(model.is_valid_var_id_str("c"));

        // add list of variables at once
        let variables = vec![("aaa", "aaa_name"), ("bbb", "bbb_name")];
        model.add_multiple_variables(variables).unwrap();
        assert_eq!(model.num_vars(), 5);
        assert!(model.is_valid_var_id_str("bbb"));
    }

    /// Test adding uninterpreted functions (both incrementally and at once).
    #[test]
    fn test_adding_uninterpreted_fns() {
        let mut model = ModelState::new_empty();

        model
            .add_empty_uninterpreted_fn_by_str("f", "f", 1)
            .unwrap();
        model
            .add_empty_uninterpreted_fn_by_str("g", "g", 0)
            .unwrap();
        let f_id = model.get_uninterpreted_fn_id("f").unwrap();
        let f = model.get_uninterpreted_fn(&f_id).unwrap();
        assert_eq!(model.num_uninterpreted_fns(), 2);
        assert_eq!(model.highest_uninterpreted_fn_arity(), 1);
        assert!(model.is_valid_uninterpreted_fn_id_str("f"));
        assert!(model.is_valid_uninterpreted_fn_id_str("g"));
        assert_eq!(f.get_arity(), 1);

        // add list of function symbols at once
        let uninterpreted_fns = vec![("ff", "ff", 4), ("gg", "gg", 3)];
        model
            .add_multiple_uninterpreted_fns(uninterpreted_fns)
            .unwrap();
        assert_eq!(model.highest_uninterpreted_fn_arity(), 4);
        assert!(model.is_valid_uninterpreted_fn_id_str("ff"));
        assert!(model.is_valid_uninterpreted_fn_id_str("gg"));
    }

    /// Test adding regulations (both incrementally and at once).
    #[test]
    fn test_adding_regulations() {
        let var_id_name_pairs = vec![("a", "a"), ("b", "b"), ("c", "c")];
        let mut model1 = ModelState::new_from_vars(var_id_name_pairs).unwrap();
        let mut model2 = model1.clone();

        // one by one, add regulations a -> a, a -> b, b -> c, c -> a
        model1.add_regulation_by_str("a -> a").unwrap();
        model1.add_regulation_by_str("a -> b").unwrap();
        model1.add_regulation_by_str("b -> c").unwrap();
        model1.add_regulation_by_str("c -> a").unwrap();
        assert_eq!(model1.num_regulations(), 4);

        // add the same regulations, but now all at once
        let regulations = vec!["a -> a", "a -> b", "b -> c", "c -> a"];
        model2.add_multiple_regulations(regulations).unwrap();

        assert_eq!(model1, model2);
    }

    /// Test manually creating `ModelState` and mutating it by adding/removing variables
    /// or regulations. We are only adding valid regulations/variables here, invalid insertions are
    /// covered by other tests.
    #[test]
    fn test_manually_editing_regulation_graph() {
        let mut model = ModelState::new_empty();

        // add variables a, b, c
        let variables = vec![("a", "a_name"), ("b", "b_name"), ("c", "c_name")];
        model.add_multiple_variables(variables).unwrap();
        assert_eq!(model.num_vars(), 3);
        assert!(model.is_valid_var_id_str("c"));

        // add regulations a -> a, a -> b, b -> c, c -> a
        let regulations = vec!["a -> a", "a -> b", "b -> c", "c -> a"];
        model.add_multiple_regulations(regulations).unwrap();
        assert_eq!(model.num_regulations(), 4);

        // remove variable and check that all its regulations disappear, try re-adding it then
        let var_a = model.get_var_id("a").unwrap();
        model.remove_var(&var_a).unwrap();
        assert!(model.get_var_name(&var_a).is_err());
        let var_b = model.get_var_id("b").unwrap();
        assert!(model.get_regulation(&var_a, &var_b).is_err());
        assert_eq!(model.num_regulations(), 1);
        assert!(model.add_var(var_a, "name_a", "").is_ok());

        // test removing a regulation and then re-adding it again
        model.remove_regulation_by_str("b -> c").unwrap();
        let var_c = model.get_var_id("c").unwrap();
        assert!(model.get_regulation(&var_b, &var_c).is_err());
        assert!(model.add_regulation_by_str("b -> c").is_ok());
        assert_eq!(model.num_regulations(), 1);
    }

    /// Test manually creating `ModelState` and mutating it by adding/removing uninterpreted fns.
    #[test]
    fn test_manually_editing_uninterpreted_fns() {
        let mut model = ModelState::new_empty();
        let uninterpreted_fns = vec![("f", "f", 4), ("g", "g", 1)];
        model
            .add_multiple_uninterpreted_fns(uninterpreted_fns)
            .unwrap();

        // test default field values of an uninterpreted fn
        let f_id = model.get_uninterpreted_fn_id("f").unwrap();
        let f = model.get_uninterpreted_fn(&f_id).unwrap();
        assert_eq!(model.num_uninterpreted_fns(), 2);
        assert_eq!(f.get_fn_expression(), "");
        assert_eq!(f.get_essential(0), &Essentiality::Unknown);
        assert_eq!(f.get_monotonic(2), &Monotonicity::Unknown);
        assert_eq!(model.highest_uninterpreted_fn_arity(), 4);

        // test setting all the various fields of an uninterpreted fn (including decreasing arity)
        model.set_uninterpreted_fn_name(&f_id, "ff").unwrap();
        model.set_uninterpreted_fn_arity(&f_id, 3).unwrap();
        model
            .set_uninterpreted_fn_essentiality(&f_id, Essentiality::True, 0)
            .unwrap();
        model
            .set_uninterpreted_fn_monotonicity(&f_id, Monotonicity::Activation, 2)
            .unwrap();
        model
            .set_uninterpreted_fn_expression(&f_id, "g(var0) | var2")
            .unwrap();
        let f = model.get_uninterpreted_fn(&f_id).unwrap();
        assert_eq!(f.get_name(), "ff");
        assert_eq!(f.get_fn_expression(), "g(var0) | var2");
        assert_eq!(f.get_essential(0), &Essentiality::True);
        assert_eq!(f.get_monotonic(2), &Monotonicity::Activation);
        assert_eq!(model.highest_uninterpreted_fn_arity(), 3);

        // function `g` cannot be removed, and its arity cannot be changed, since it is used inside the
        // expression for function `f`
        let remove_error = model.remove_uninterpreted_fn_by_str("g");
        assert!(remove_error.is_err());
        let modify_arity_error = model.set_uninterpreted_fn_arity_by_str("g", 2);
        assert!(modify_arity_error.is_err());

        // test setting ID of an uninterpreted fn
        model.set_uninterpreted_fn_id_by_str("g", "h").unwrap();
        let f = model.get_uninterpreted_fn(&f_id).unwrap();
        assert_eq!(f.get_fn_expression(), "h(var0) | var2");

        // test editing function's expression back to an empty string
        model.set_uninterpreted_fn_expression(&f_id, "").unwrap();
        let f = model.get_uninterpreted_fn(&f_id).unwrap();
        assert_eq!(f.get_fn_expression(), "");

        // test increasing arity (the function symbol is no longer used in any expression)
        model.set_uninterpreted_fn_arity_by_str("h", 5).unwrap();
        assert_eq!(model.highest_uninterpreted_fn_arity(), 5);

        // test removing function with NOT the most arguments (should not influence the
        // highest uninterpreted fn arity
        model.remove_uninterpreted_fn(&f_id).unwrap();
        assert_eq!(model.num_uninterpreted_fns(), 1);
        assert_eq!(model.highest_uninterpreted_fn_arity(), 5);

        // test removing function with the most arguments (should lower the highest
        // uninterpreted fn arity
        model.remove_uninterpreted_fn_by_str("h").unwrap();
        assert_eq!(model.num_uninterpreted_fns(), 0);
        assert_eq!(model.highest_uninterpreted_fn_arity(), 0);
    }

    /// Test manually adding and modifying update functions.
    #[test]
    fn test_update_fns() {
        let var_id_name_pairs = vec![("a", "a"), ("b", "b"), ("c", "c")];
        let mut model = ModelState::new_from_vars(var_id_name_pairs).unwrap();
        let var_a = model.get_var_id("a").unwrap();

        let initial_expression = model.get_update_fn_string(&var_a).unwrap();
        assert_eq!(initial_expression, "");

        let expression = "(a & b) => c";
        model.set_update_fn(&var_a, "(a & b) => c").unwrap();
        let modified_expression = model.get_update_fn_string(&var_a).unwrap();
        assert_eq!(modified_expression, expression);
    }

    /// Test adding invalid variables.
    #[test]
    fn test_add_invalid_vars() {
        let mut model = ModelState::new_empty();
        // same names should not be an issue
        let variables = vec![("a", "a_name"), ("b", "b_name")];
        model.add_multiple_variables(variables).unwrap();
        assert_eq!(model.num_vars(), 2);

        // adding same ID again should cause error
        assert!(model.add_var_by_str("a", "whatever", "").is_err());
        // adding invalid ID string should cause error
        assert!(model.add_var_by_str("a ", "whatever2", "").is_err());
        assert!(model.add_var_by_str("(", "whatever3", "").is_err());
        assert!(model.add_var_by_str("aa+a", "whatever4", "").is_err());

        assert_eq!(model.num_vars(), 2);
    }

    /// Test adding invalid regulations.
    #[test]
    fn test_add_invalid_regs() {
        let mut model = ModelState::new_empty();
        let variables = vec![("a", "a_name"), ("b", "b_name")];
        model.add_multiple_variables(variables).unwrap();
        let var_a = model.get_var_id("a").unwrap();
        let var_b = model.get_var_id("b").unwrap();

        // add one valid regulation a -> b
        model.add_regulation_by_str("a -> b").unwrap();

        // adding reg between the same two vars again should cause an error
        assert!(model.add_regulation_by_str("a -> b").is_err());
        assert!(model.add_regulation_by_str("a -| b").is_err());
        assert!(model
            .add_regulation(var_a, var_b, Essentiality::Unknown, Monotonicity::Dual)
            .is_err());

        // adding reg with invalid vars or invalid format should cause error
        assert!(model.add_regulation_by_str("a -> a b").is_err());
        assert!(model.add_regulation_by_str("X -> a").is_err());
        assert!(model.add_regulation_by_str("a -@ a").is_err());

        // check that nothing really got added
        assert_eq!(model.num_regulations(), 1);
    }

    /// Test that changing variable's name works correctly.
    #[test]
    fn test_var_name_change() {
        let mut model = ModelState::new_empty();
        let variables = vec![("a", "a_name"), ("b", "b_name")];
        model.add_multiple_variables(variables).unwrap();
        let var_a = model.get_var_id("a").unwrap();

        // setting random unique name
        model.set_var_name(&var_a, "new_name").unwrap();
        assert_eq!(model.get_var_name(&var_a).unwrap(), "new_name");

        // setting already existing name should also work
        model.set_var_name(&var_a, "b_name").unwrap();
        assert_eq!(model.get_var_name(&var_a).unwrap(), "b_name");
    }

    /// Test that changing variable's ID works correctly.
    #[test]
    fn test_var_id_change() {
        let mut model = ModelState::new_empty();
        let var_a = model.generate_var_id("a", None);
        model.add_var(var_a.clone(), "a_name", "").unwrap();
        let var_b = model.generate_var_id("b", None);
        model.add_var(var_b.clone(), "b_name", "").unwrap();

        // add regulations a -> a, a -> b, b -> a, b -> b
        let regulations = vec!["a -> a", "a -> b", "b -> a", "b -> b"];
        model.add_multiple_regulations(regulations).unwrap();

        // add layout
        let default_layout_id = ModelState::get_default_layout_id();
        let new_layout_id = model.generate_layout_id("layout2", None);
        model
            .add_layout_copy(new_layout_id.clone(), "layout2", &default_layout_id)
            .unwrap();

        // add update fn expression for both variables contain `a`
        model.set_update_fn(&var_a, "a | b").unwrap();
        model.set_update_fn(&var_b, "a => a").unwrap();

        // change var id of variable a, check that it correctly changed everywhere
        let new_var = model.generate_var_id("c", None);
        model.set_var_id(&var_a, new_var.clone()).unwrap();

        // 1) variable map changed correctly
        assert!(!model.is_valid_var_id(&var_a));
        assert!(model.is_valid_var_id(&new_var));

        // 2) regulations changed correctly
        assert_eq!(model.num_regulations(), 4);
        assert!(model.get_regulation(&new_var, &new_var).is_ok());
        assert!(model.get_regulation(&new_var, &var_b).is_ok());
        assert!(model.get_regulation(&var_b, &new_var).is_ok());
        assert!(model.get_regulation(&var_b, &var_b).is_ok());

        // 3) layouts changed correctly
        // we check the layout objects directly, because ModelState shorthands would fail
        // automatically because the variable is no longer valid for the ModelState
        let layout1 = model.get_layout(&default_layout_id).unwrap();
        let layout2 = model.get_layout(&new_layout_id).unwrap();
        assert!(layout1.get_node_position(&var_a).is_err());
        assert!(layout2.get_node_position(&var_a).is_err());
        assert!(layout1.get_node_position(&new_var).is_ok());
        assert!(layout2.get_node_position(&new_var).is_ok());

        // 4) update functions changed correctly
        assert!(model.get_update_fn_string(&var_a).is_err());
        assert_eq!(model.get_update_fn_string(&new_var).unwrap(), "c | b");
        assert_eq!(model.get_update_fn_string(&var_b).unwrap(), "c => c");
    }

    #[test]
    fn test_layout_manipulation() {
        let var_id_name_pairs = vec![("a_id", "a_name"), ("b_id", "b_name")];
        let mut model = ModelState::new_from_vars(var_id_name_pairs).unwrap();
        assert_eq!(model.num_layouts(), 1);

        // check default layout
        let default_layout_id = ModelState::get_default_layout_id();
        let default_layout_name = ModelState::get_default_layout_name();
        let default_layout = model.get_layout(&default_layout_id).unwrap();
        assert!(model.is_valid_layout_id(&default_layout_id));
        assert_eq!(default_layout.get_num_nodes(), 2);
        assert_eq!(default_layout.get_layout_name(), default_layout_name);

        // change default layout's nodes
        let var_id = model.get_var_id("a_id").unwrap();
        model
            .update_position(&default_layout_id, &var_id, 2., 2.)
            .unwrap();
        let position = model
            .get_node_position(&default_layout_id, &var_id)
            .unwrap();
        assert_eq!(position, &NodePosition(2., 2.));

        // add layouts (one as vars with default nodes, and other as direct copy)
        let new_id_1 = model.generate_layout_id("new_layout", None);
        model
            .add_layout_simple(new_id_1.clone(), "new_layout")
            .unwrap();
        let position = model.get_node_position(&new_id_1, &var_id).unwrap();
        assert_eq!(position, &NodePosition(0., 0.));

        let new_id_2 = model.generate_layout_id("another_layout", None);
        model
            .add_layout_copy(new_id_2.clone(), "new_layout", &default_layout_id)
            .unwrap();
        let position = model.get_node_position(&new_id_2, &var_id).unwrap();
        assert_eq!(position, &NodePosition(2., 2.));
    }
}
