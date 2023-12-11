use crate::sketchbook::{LayoutId, ModelState, VarId};

/// Methods for safely generating valid instances of identifiers for the current `ModelState`.
impl ModelState {
    /// Generate valid `VarId` that's currently not used by any variable in this `ModelState`.
    ///
    /// First, the variable's name or its transformation by replacing invalid characters are tried.
    /// If they are both invalid (non-unique), a numerical identifier is added at the end.
    ///
    /// **Warning:** Do not use this to pre-generate more than one id, as it is deterministic.
    pub fn generate_var_id(&self, var_name: &str) -> VarId {
        // first try to generate the id using the given name
        if let Ok(var_id) = VarId::new(var_name) {
            // the id must not be valid in the network already (that would mean it is already used)
            if !self.is_valid_var_id(&var_id) {
                return var_id;
            }
        }

        // try to transform the name by removing invalid characters
        let transformed_name = Self::transform_to_id(var_name);
        if let Ok(var_id) = VarId::new(transformed_name.as_str()) {
            // the id must not be valid in the network already (that would mean it is already used)
            if !self.is_valid_var_id(&var_id) {
                return var_id;
            }
        }

        // finally, append a number after the name
        // start searching at 0, until we try `self.num_vars()` options
        for n in 0..self.num_vars() {
            let var_id = VarId::new(format!("{}_{}", transformed_name, n).as_str()).unwrap();
            if !self.is_valid_var_id(&var_id) {
                return var_id;
            }
        }

        // this must be valid, we already tried more than self.num_vars() options
        VarId::new(format!("{}_{}", transformed_name, self.num_vars()).as_str()).unwrap()
    }

    /// Generate valid `LayoutId` that's currently not used by layouts in this `ModelState`.
    ///
    /// First, the variable's name or its transformation by replacing invalid characters are tried.
    /// If they are both invalid (non-unique), a numerical identifier is added at the end.
    ///
    /// **Warning:** Do not use this to pre-generate more than one id, as it is deterministic.
    pub fn generate_layout_id(&self, layout_name: &str) -> LayoutId {
        // first try to generate the id using the name
        if let Ok(layout_id) = LayoutId::new(layout_name) {
            // the id must not be valid in the network already (that would mean it is already used)
            if !self.is_valid_layout_id(&layout_id) {
                return layout_id;
            }
        }

        // try to transform the name by removing invalid characters
        let transformed_name = Self::transform_to_id(layout_name);
        if let Ok(layout_id) = LayoutId::new(transformed_name.as_str()) {
            // the id must not be valid in the network already (that would mean it is already used)
            if !self.is_valid_layout_id(&layout_id) {
                return layout_id;
            }
        }

        // finally, append a number after the name
        // start searching at 0, until we try `self.num_layouts()` options
        for n in 0..self.num_layouts() {
            let layout_id = LayoutId::new(format!("{}_{}", transformed_name, n).as_str()).unwrap();
            if !self.is_valid_layout_id(&layout_id) {
                return layout_id;
            }
        }

        // this must be valid, we already tried more than self.num_layouts() options
        LayoutId::new(format!("{}_{}", transformed_name, self.num_layouts()).as_str()).unwrap()
    }

    /// **(internal)** Transform a string to an identifier string by removing all the
    /// invalid characters.
    fn transform_to_id(random_str: &str) -> String {
        random_str
            .chars()
            .filter(|ch| ch.is_alphanumeric() || *ch == '_')
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::layout::LayoutId;
    use crate::sketchbook::{ModelState, VarId};

    #[test]
    fn test_id_transformation() {
        // string slice that is a valid identifier as is
        let string1 = "id_123";
        // string slice that is not a valid identifier as it contains various invalid characters
        let string2 = "i<d ??_12)&3    ";

        let transformed1 = ModelState::transform_to_id(string1);
        let transformed2 = ModelState::transform_to_id(string2);
        assert_eq!(transformed1, transformed2);
        assert_eq!(transformed1, "id_123".to_string());
    }

    #[test]
    fn test_var_id_generating() {
        let reg_state =
            ModelState::new_from_vars(vec![("a", "name"), ("b", "name"), ("c", "name")]).unwrap();
        assert_eq!(reg_state.num_vars(), 3);

        // name slice that is a valid identifier as is
        let var_name_1 = "d";
        assert_eq!(
            reg_state.generate_var_id(var_name_1),
            VarId::new("d").unwrap()
        );

        // name that is not a valid identifier as it contains various invalid characters
        let var_name_2 = "-d ??)&    ";
        assert_eq!(
            reg_state.generate_var_id(var_name_2),
            VarId::new("d").unwrap()
        );

        // name that is already used in the network
        let var_name_3 = "a";
        // result will contain an numerical index in the end
        assert_eq!(
            reg_state.generate_var_id(var_name_3),
            VarId::new("a_0").unwrap()
        );
    }

    #[test]
    fn test_layout_id_generating() {
        let mut reg_state = ModelState::new();
        let layout_id = LayoutId::new("l_0").unwrap();
        let default_layout_id = ModelState::get_default_layout_id();
        reg_state
            .add_layout_copy(layout_id, "name", &default_layout_id)
            .unwrap();
        assert_eq!(reg_state.num_layouts(), 2);

        // expected result for all the following IDs will be the same
        let expected = LayoutId::new("l_1").unwrap();

        // name slice that is a valid identifier as is
        let name_1 = "l_1";
        assert_eq!(reg_state.generate_layout_id(name_1), expected);

        // name that is not a valid identifier as it contains various invalid characters
        let name_2 = "%%%%l_    1)";
        assert_eq!(reg_state.generate_layout_id(name_2), expected);

        // add new layout
        let layout_id = LayoutId::new("l").unwrap();
        reg_state
            .add_layout_copy(layout_id, "name", &default_layout_id)
            .unwrap();

        // try generate ID for the same layout again - the result will have numerical index appended
        // however, this time we cant just add index 0 because the result would not be unique

        let name_3 = "l";
        // search for unused index is incremental, starting at 0 (until valid index 1 is found)
        assert_eq!(reg_state.generate_layout_id(name_3), expected);
    }
}
