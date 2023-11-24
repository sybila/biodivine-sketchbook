use crate::sketchbook::{LayoutId, RegulationsState, VarId};

/// Methods for safely generating valid instances of identifiers for the current `RegulationsState`.
impl RegulationsState {
    /// Generate valid `VarId` that's currently not used by any variable in this `RegulationsState`.
    ///
    /// First, the variable's name or its transformation are tried. If they are both invalid,
    /// a numerical identifier is constructed.
    ///
    /// **warning** Do not use this to pre-generate more than one id.
    pub fn generate_var_id(&self, var_name: &str) -> VarId {
        // TODO: how to generate the valid VarId properly?

        // first try to generate the id using the name
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

        // finally using the numeric ID, starting at `n_vars`, until we find valid one
        let n_vars = self.num_vars();
        for n in 0..n_vars {
            // valid identifier string
            let var_id = VarId::new(format!("v_{}", n_vars + n).as_str()).unwrap();
            if !self.is_valid_var_id(&var_id) {
                return var_id;
            }
        }

        // this must be valid, we already tried more than self.num_vars() options
        VarId::new(format!("v_{}", n_vars * 2).as_str()).unwrap()
    }

    /// Generate valid `LayoutId` that's currently not used by layouts in this `RegulationsState`.
    ///
    /// First, the layout's name or its transformation are tried. If they are both invalid,
    /// a numerical identifier is constructed.
    ///
    /// **warning** Do not use this to pre-generate more than one id.
    pub fn generate_layout_id(&self, layout_name: &str) -> LayoutId {
        // TODO: how to generate the valid LayoutId properly?

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

        // finally using the numeric ID, starting at `n_layouts`, until we find valid one
        let n_layouts = self.num_layouts();
        for n in 0..n_layouts {
            // valid identifier string
            let layout_id = LayoutId::new(format!("l_{}", n_layouts + n).as_str()).unwrap();
            if !self.is_valid_layout_id(&layout_id) {
                return layout_id;
            }
        }

        // this must be valid, we already tried more than self.num_layouts() options
        LayoutId::new(format!("l_{}", n_layouts * 2).as_str()).unwrap()
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
    use crate::sketchbook::{RegulationsState, VarId};

    #[test]
    fn test_id_transformation() {
        // string slice that is a valid identifier as is
        let string1 = "id_123";
        // string slice that is not a valid identifier as it contains various invalid characters
        let string2 = "i<d ??_12)&3    ";

        let transformed1 = RegulationsState::transform_to_id(string1);
        let transformed2 = RegulationsState::transform_to_id(string2);
        assert_eq!(transformed1, transformed2);
        assert_eq!(transformed1, "id_123".to_string());
    }

    #[test]
    fn test_var_id_generating() {
        let reg_state =
            RegulationsState::new_from_vars(vec![("a0", "name"), ("a1", "name"), ("a2", "name")])
                .unwrap();
        assert_eq!(reg_state.num_vars(), 3);

        // name slice that is a valid identifier as is
        let var_name_1 = "a3";
        assert_eq!(
            reg_state.generate_var_id(var_name_1),
            VarId::new("a3").unwrap()
        );

        // name that is not a valid identifier as it contains various invalid characters
        let var_name_2 = "a ??)&4    ";
        assert_eq!(
            reg_state.generate_var_id(var_name_2),
            VarId::new("a4").unwrap()
        );

        // name that is already used in the network
        let var_name_3 = "a1";
        // result will be "v_" + numerical index
        // search for unused index is incremental and starts at <CURRENT_NUM_OF_VARS>
        assert_eq!(
            reg_state.generate_var_id(var_name_3),
            VarId::new("v_3").unwrap()
        );
    }

    #[test]
    fn test_layout_id_generating() {
        let mut reg_state = RegulationsState::new();
        let layout_id = LayoutId::new("l_1").unwrap();
        let default_layout_id = RegulationsState::get_default_layout_id();
        reg_state
            .add_layout_copy(layout_id, "name", &default_layout_id)
            .unwrap();
        assert_eq!(reg_state.num_layouts(), 2);

        // expected result for all the following IDs will be the same
        let expected = LayoutId::new("l_2").unwrap();

        // name slice that is a valid identifier as is
        let name_1 = "l_2";
        assert_eq!(reg_state.generate_layout_id(name_1), expected);

        // name that is not a valid identifier as it contains various invalid characters
        let name_2 = "%%%%l_    2)";
        assert_eq!(reg_state.generate_layout_id(name_2), expected);

        // name that is already used in the network
        let name_3 = "l_1";
        // result will be "l_" + numerical index
        // search for unused index is incremental and starts at <CURRENT_NUM_OF_LAYOUTS>
        assert_eq!(reg_state.generate_layout_id(name_3), expected);
    }
}
