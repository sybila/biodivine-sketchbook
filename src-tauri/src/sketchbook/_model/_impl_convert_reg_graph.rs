use crate::sketchbook::{Essentiality, ModelState, Monotonicity, VarId};
use biodivine_lib_param_bn::Monotonicity as Lib_Pbn_Monotonicity;
use biodivine_lib_param_bn::RegulatoryGraph;

/// Methods for converting between `ModelState` and `RegulatoryGraph` (from the `lib-param-bn`).
impl ModelState {
    /// Convert the `ModelState` (the current state of the regulation graph) into the
    /// corresponding `RegulatoryGraph` object. Sorted variable IDs of the `ModelState` are
    /// used for variable names in `RegulatoryGraph`.
    ///
    /// The conversion might loose some information, as the `RegulatoryGraph` does not support
    /// all the variants of `Monotonicity` and `Essentiality`. See also [ModelState::sign_to_monotonicity].
    ///
    /// Note that we can convert the resulting `RegulatoryGraph` back, but the conversion loses
    /// some information, like the original variable names and layout information.
    /// Also, all of the other model components, such as `update functions` or `uninterpreted functions`
    /// are not part of the `RegulatoryGraph`.
    pub fn to_reg_graph(&self) -> RegulatoryGraph {
        // create `RegulatoryGraph` from a list of variable ID strings (these are unique and
        // can be mapped back)
        let mut variable_vec = self
            .variables
            .keys()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        // sort the IDs, so that the process is kinda deterministic - in `RegulatoryGraph`, the
        // order of the variables matters (but regulations order does not)
        variable_vec.sort();
        let mut reg_graph = RegulatoryGraph::new(variable_vec);

        // regulations
        for r in self.regulations() {
            reg_graph
                .add_regulation(
                    r.get_regulator().as_str(),
                    r.get_target().as_str(),
                    r.is_essential(),
                    ModelState::sign_to_monotonicity(r.get_sign()),
                )
                .unwrap();
            // we can use unwrap, cause the regulation will always be unique and correctly added
        }
        reg_graph
    }

    /// Convert the `RegulatoryGraph` into the corresponding `ModelState` object. A name
    /// of the variable used in `RegulatoryGraph` (which should be unique) is used as both its ID
    /// and name in the resulting `ModelState`.
    ///
    /// Note that only the default layout (all nodes at 0,0) is created for the `ModelState`.
    pub fn from_reg_graph(reg_graph: RegulatoryGraph) -> Result<ModelState, String> {
        let mut model = ModelState::new();

        // variables
        for v in reg_graph.variables() {
            // name in the `RegulatoryGraph` is a unique valid identifier
            let name_in_graph = reg_graph.get_variable_name(v);
            model.add_var(VarId::new(name_in_graph.as_str())?, name_in_graph)?;
        }

        // regulations
        for r in reg_graph.regulations() {
            let name_regulator = reg_graph.get_variable_name(r.get_regulator());
            let name_target = reg_graph.get_variable_name(r.get_target());
            model.add_regulation(
                VarId::new(name_regulator.as_str())?,
                VarId::new(name_target.as_str())?,
                ModelState::essentiality_from_bool(r.is_observable()),
                ModelState::sign_from_monotonicity(r.get_monotonicity()),
            )?;
        }
        Ok(model)
    }

    /// **(internal)** Static utility method to convert regulation sign given by `Monotonicity`
    /// used by `lib_param_bn` into the type `Monotonicity` used here.
    /// TODO: note that `lib-param-bn` currently cannot express `Dual` variant of `Monotonicity`.
    fn sign_from_monotonicity(monotonicity: Option<Lib_Pbn_Monotonicity>) -> Monotonicity {
        match monotonicity {
            Some(m) => match m {
                Lib_Pbn_Monotonicity::Activation => Monotonicity::Activation,
                Lib_Pbn_Monotonicity::Inhibition => Monotonicity::Inhibition,
            },
            None => Monotonicity::Unknown,
        }
    }

    /// **(internal)** Static utility method to convert regulation sign from the type
    /// `Monotonicity` used here into the type `Monotonicity` used in `lib_param_bn`.
    /// TODO: note that `lib-param-bn` currently cannot express `Dual` variant of `Monotonicity` and `Unknown` is used instead.
    fn sign_to_monotonicity(regulation_sign: &Monotonicity) -> Option<Lib_Pbn_Monotonicity> {
        match regulation_sign {
            Monotonicity::Activation => Some(Lib_Pbn_Monotonicity::Activation),
            Monotonicity::Inhibition => Some(Lib_Pbn_Monotonicity::Inhibition),
            Monotonicity::Unknown => None,
            // todo: fix
            Monotonicity::Dual => None,
        }
    }

    /// **(internal)** Static utility method to convert `Essentiality` from boolean.
    /// TODO: note that `lib-param-bn` currently cannot distinguish between `False` and `Unknown` variants of `Essentiality`.
    fn essentiality_from_bool(essentiality: bool) -> Essentiality {
        match essentiality {
            true => Essentiality::True,
            // todo: fix, this is how it works now in `lib-param-bn`
            false => Essentiality::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::ModelState;
    use biodivine_lib_param_bn::RegulatoryGraph;

    /// Prepare a test model containing only variables and regulations.
    fn prepare_test_model() -> ModelState {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
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

        let model_back = ModelState::from_reg_graph(reg_graph).unwrap();
        assert_eq!(model, model_back);
    }

    #[test]
    fn test_from_reg_graph() {
        let mut reg_graph = RegulatoryGraph::new(vec!["a".to_string(), "b".to_string()]);
        reg_graph.add_string_regulation("a -> b").unwrap();
        reg_graph.add_string_regulation("b -> a").unwrap();

        let model = ModelState::from_reg_graph(reg_graph.clone()).unwrap();
        assert_eq!(model.num_vars(), 2);

        let reg_graph_back = model.to_reg_graph();
        assert_eq!(reg_graph, reg_graph_back);
    }
}
