use crate::sketchbook::bn_utils;
use crate::sketchbook::ids::VarId;
use crate::sketchbook::model::ModelState;
use biodivine_lib_param_bn::RegulatoryGraph;

/// Methods for converting between `ModelState` and `RegulatoryGraph` (from the `lib-param-bn`).
impl ModelState {
    /// Extract the regulatory graph (`RegulatoryGraph` object) from the `ModelState` instance.
    /// Sorted string variable IDs of the `ModelState` are used for variable names in the resulting
    /// `RegulatoryGraph`. Internal numerical IDs are generated from the variable order.
    ///
    /// The conversion might loose some information, as the `RegulatoryGraph` does not support
    /// all the variants of `Monotonicity` and `Essentiality`. See also [bn_utils::sign_to_monotonicity].
    /// Metadata like original variable annotations or names, and layout positions are lost.
    ///
    /// Also note that other model components like update or uninterpreted functions are not part
    /// of the `RegulatoryGraph`. See [Self::to_bn] instead if you want a whole BN.
    pub fn to_reg_graph(&self) -> RegulatoryGraph {
        self._to_reg_graph(true)
    }

    /// Extract the regulatory graph (`RegulatoryGraph` object) from the `ModelState` instance.
    /// Sorted string variable IDs of the `ModelState` are used for variable names in the resulting
    /// `RegulatoryGraph`. Internal numerical IDs are generated from the variable order.
    ///
    /// The types of regulations (both monotonicity and essentiality) are intentionally ignored. This
    /// is useful when we want to process regulation characteristics separately via static properties.
    pub fn to_reg_graph_with_unspecified_regs(&self) -> RegulatoryGraph {
        self._to_reg_graph(false)
    }

    /// Internal utility to extract a `RegulatoryGraph` object from the `ModelState` instance.
    ///
    /// There are two modes based on `include_reg_types` argument. If set to `true`, the
    /// types of regulations (their essentiality and monotonicity) are preserved. Otherwise,
    /// they are ignored, and regulations of unspecified type are used instead.
    fn _to_reg_graph(&self, include_reg_types: bool) -> RegulatoryGraph {
        // Create `RegulatoryGraph` from a list of variable ID strings (these are unique and
        // can be mapped back)
        let mut variable_vec = self
            .variables
            .keys()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();

        // Sort the IDs, so that the process is kinda deterministic - in `RegulatoryGraph`, the
        // order of the variables matters (but regulations order does not)
        variable_vec.sort();
        let mut reg_graph = RegulatoryGraph::new(variable_vec);

        // Add all the regulations accordingly
        for r in self.regulations() {
            if include_reg_types {
                // Add the regulation with its original characteristics (as far as conversion allows)
                // We can safely unwrap cause the regulation is ensured to be unique and valid
                reg_graph
                    .add_regulation(
                        r.get_regulator().as_str(),
                        r.get_target().as_str(),
                        r.is_essential(),
                        bn_utils::sign_to_monotonicity(r.get_sign()),
                    )
                    .unwrap();
            } else {
                // Add the regulation with unspecified monotonicity and essentiality
                // We can safely unwrap cause the regulation is ensured to be unique and valid
                reg_graph
                    .add_regulation(
                        r.get_regulator().as_str(),
                        r.get_target().as_str(),
                        false,
                        None,
                    )
                    .unwrap();
            }
        }
        reg_graph
    }

    /// Create a `ModelState` from a given `RegulatoryGraph` instance. The model will have
    /// the provided variables and regulations. Name of each variable (and uninterpreted fn)
    /// used in the `RegulatoryGraph` (which should be unique) is used as both its ID and name
    /// in the resulting `ModelState`.
    ///
    /// All other components of the `ModelState` not present in regulatory graph (update
    /// functions, uninterpreted functions,...) are left empty. A default layout (all nodes
    /// at origin 0,0) is created for the variables. Annotations are left empty.
    pub fn from_reg_graph(reg_graph: &RegulatoryGraph) -> Result<ModelState, String> {
        let mut model = ModelState::new_empty();

        // Add the variables first
        for v in reg_graph.variables() {
            // The name used in the `RegulatoryGraph` is a unique and valid string identifier
            let name_in_graph = reg_graph.get_variable_name(v);
            model.add_var(VarId::new(name_in_graph.as_str())?, name_in_graph, "")?;
        }

        // Add the regulations
        for r in reg_graph.regulations() {
            let name_regulator = reg_graph.get_variable_name(r.get_regulator());
            let name_target = reg_graph.get_variable_name(r.get_target());
            model.add_regulation(
                VarId::new(name_regulator.as_str())?,
                VarId::new(name_target.as_str())?,
                bn_utils::essentiality_from_bool(r.is_observable()),
                bn_utils::sign_from_monotonicity(r.get_monotonicity()),
            )?;
        }
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::model::ModelState;
    use biodivine_lib_param_bn::RegulatoryGraph;

    /// Prepare a test model containing only variables and regulations.
    fn prepare_test_model() -> ModelState {
        let mut model = ModelState::new_with_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model
            .add_multiple_regulations(vec!["a -> b", "b -> a", "a -| a"])
            .unwrap();
        model
    }

    #[test]
    fn test_to_reg_graph() {
        let model = prepare_test_model();
        let reg_graph = model.to_reg_graph();
        let var_a = reg_graph.find_variable("a").unwrap();
        let var_b = reg_graph.find_variable("b").unwrap();
        assert_eq!(reg_graph.num_vars(), 2);
        assert_eq!(reg_graph.regulators(var_a), vec![var_a, var_b]);

        let model_back = ModelState::from_reg_graph(&reg_graph).unwrap();
        assert_eq!(model, model_back);
    }

    #[test]
    fn test_from_reg_graph() {
        let mut reg_graph = RegulatoryGraph::new(vec!["a".to_string(), "b".to_string()]);
        reg_graph.add_string_regulation("a -> b").unwrap();
        reg_graph.add_string_regulation("b -> a").unwrap();

        let model = ModelState::from_reg_graph(&reg_graph).unwrap();
        assert_eq!(model.num_vars(), 2);

        let reg_graph_back = model.to_reg_graph();
        assert_eq!(reg_graph, reg_graph_back);
    }
}
