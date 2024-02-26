use crate::sketchbook::{LayoutId, ModelState, UninterpretedFnId, VarId};
use std::str::FromStr;

/// Methods for safely generating valid instances of identifiers for the current `ModelState`.
impl ModelState {
    /// Generate valid `VarId` that's currently not used by any variable in this `ModelState`.
    ///
    /// First, the variable's name or its transformation by replacing invalid characters are tried.
    /// If they are both invalid (non-unique), a numerical identifier is added at the end.
    ///
    /// **Warning:** Do not use this to pre-generate more than one id at a time, as the process
    /// is deterministic and might generate the same IDs. Always generate an Id, add that variable to
    /// the model, and then repeat for other variables.
    pub fn generate_var_id(&self, var_name: &str) -> VarId {
        self.generate_id(var_name, &(Self::is_valid_var_id), self.num_vars())
    }

    /// Generate valid `LayoutId` that's currently not used by layouts in this `ModelState`.
    ///
    /// First, the variable's name or its transformation by replacing invalid characters are tried.
    /// If they are both invalid (non-unique), a numerical identifier is added at the end.
    ///
    /// **Warning:** Do not use this to pre-generate more than one id at a time, as the process
    /// is deterministic and might generate the same IDs. Always generate an Id, add that layout to
    /// the model, and then repeat for other layouts.
    pub fn generate_layout_id(&self, layout_name: &str) -> LayoutId {
        self.generate_id(layout_name, &(Self::is_valid_layout_id), self.num_layouts())
    }

    /// Generate valid `UninterpretedFnId` that's currently not used by uninterpreted_fns in this `ModelState`.
    ///
    /// First, the uninterpreted fn's name or its transformation by replacing invalid characters are tried.
    /// If they are both invalid (non-unique), a numerical identifier is added at the end.
    ///
    /// **Warning:** Do not use this to pre-generate more than one id at a time, as the process
    /// is deterministic and might generate the same IDs. Always generate an Id, add that fn to
    /// the model, and then repeat for other fns.
    pub fn generate_uninterpreted_fn_id(&self, fn_name: &str) -> UninterpretedFnId {
        self.generate_id(
            fn_name,
            &(Self::is_valid_uninterpreted_fn_id),
            self.num_uninterpreted_fns(),
        )
    }

    /// Generate an ID of type `T` for a component of a `model` (e.g., generate a `VariableId`
    /// for a `Variable`), given the component's name, a method to check if a generated id is
    /// taken, and maximum index that the object can take (e.g., for a variable use total number
    /// of variables in the model).
    fn generate_id<T>(&self, name: &str, is_taken: &dyn Fn(&Self, &T) -> bool, max_idx: usize) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        // first try to generate the id using the given name
        if let Ok(id) = T::from_str(name) {
            // the id must not be valid in the network already (that would mean it is already used)
            if !is_taken(self, &id) {
                return id;
            }
        }

        // try to transform the name by removing invalid characters
        let transformed_name: String = name
            .chars()
            .filter(|ch| ch.is_alphanumeric() || *ch == '_')
            .collect();

        if let Ok(id) = T::from_str(transformed_name.as_str()) {
            // the id must not be valid in the network already (that would mean it is already used)
            if !is_taken(self, &id) {
                return id;
            }
        }

        // finally, append a number after the name
        // start searching at 0, until we try `max_idx` options
        for n in 0..max_idx {
            let id = T::from_str(format!("{}_{}", transformed_name, n).as_str()).unwrap();
            if !is_taken(self, &id) {
                return id;
            }
        }

        // this must be valid, we already tried more than `max_idx` options
        T::from_str(format!("{}_{}", transformed_name, max_idx).as_str()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::layout::LayoutId;
    use crate::sketchbook::{ModelState, VarId};

    #[test]
    fn test_var_id_generating() {
        let model =
            ModelState::new_from_vars(vec![("a", "name"), ("b", "name"), ("c", "name")]).unwrap();
        assert_eq!(model.num_vars(), 3);

        // name slice that is a valid identifier as is
        let var_name_1 = "d";
        assert_eq!(model.generate_var_id(var_name_1), VarId::new("d").unwrap());

        // name that is not a valid identifier as it contains various invalid characters
        let var_name_2 = "-d ??)&    ";
        assert_eq!(model.generate_var_id(var_name_2), VarId::new("d").unwrap());

        // name that is already used in the network
        let var_name_3 = "a";
        // result will contain an numerical index in the end
        assert_eq!(
            model.generate_var_id(var_name_3),
            VarId::new("a_0").unwrap()
        );
    }

    #[test]
    fn test_layout_id_generating() {
        let mut model = ModelState::new();
        let layout_id = LayoutId::new("l_0").unwrap();
        let default_layout_id = ModelState::get_default_layout_id();
        model.add_layout_simple(layout_id, "name").unwrap();
        assert_eq!(model.num_layouts(), 2);

        // expected result for all the following IDs will be the same
        let expected = LayoutId::new("l_1").unwrap();

        // name slice that is a valid identifier as is
        let name_1 = "l_1";
        assert_eq!(model.generate_layout_id(name_1), expected);

        // name that is not a valid identifier as it contains various invalid characters
        let name_2 = "%%%%l_    1)";
        assert_eq!(model.generate_layout_id(name_2), expected);

        // add new layout
        let layout_id = LayoutId::new("l").unwrap();
        model
            .add_layout_copy(layout_id, "name", &default_layout_id)
            .unwrap();

        // try generate ID for the same layout again - the result will have numerical index appended
        // however, this time we cant just add index 0 because the result would not be unique

        let name_3 = "l";
        // search for unused index is incremental, starting at 0 (until valid index 1 is found)
        assert_eq!(model.generate_layout_id(name_3), expected);
    }
}
