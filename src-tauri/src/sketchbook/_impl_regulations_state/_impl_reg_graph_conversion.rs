use crate::sketchbook::{Monotonicity, RegulationsState, VarId};
use biodivine_lib_param_bn::Monotonicity as Lib_Pbn_Monotonicity;
use biodivine_lib_param_bn::RegulatoryGraph;

/// Methods for converting between `RegulationsState` and `RegulatoryGraph` (from the lib-param-bn).
impl RegulationsState {
    /// Convert the `RegulationsState` (the current state of the regulation graph) into the
    /// corresponding `RegulatoryGraph` object. Sorted variable IDs of the `RegulationsState` are
    /// used for variable names in `RegulatoryGraph`.
    ///
    /// Note that we can convert the resulting `RegulatoryGraph` back, but the conversion loses
    /// some information, like the original variable names and layout info.
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
                    r.is_observable(),
                    RegulationsState::monotonicity_to_lib_pbn(r.get_monotonicity()),
                )
                .unwrap();
            // we can use unwrap, cause the regulation will always be unique and correctly added
        }
        reg_graph
    }

    /// Convert the `RegulatoryGraph` into the corresponding `RegulationsState` object. A name
    /// of the variable used in `RegulatoryGraph` (which should be unique) is used as both its ID
    /// and name in the resulting `RegulationsState`.
    ///
    /// Note that only the default layout (all nodes at 0,0) is created for the `RegulationsState`.
    pub fn from_reg_graph(reg_graph: RegulatoryGraph) -> Result<RegulationsState, String> {
        let mut reg_state = RegulationsState::new();

        // variables
        for v in reg_graph.variables() {
            // name in the `RegulatoryGraph` is a unique valid identifier
            let name_in_graph = reg_graph.get_variable_name(v);
            reg_state.add_var(VarId::new(name_in_graph.as_str())?, name_in_graph)?;
        }

        // regulations
        for r in reg_graph.regulations() {
            let name_regulator = reg_graph.get_variable_name(r.get_regulator());
            let name_target = reg_graph.get_variable_name(r.get_target());
            reg_state.add_regulation(
                VarId::new(name_regulator.as_str())?,
                VarId::new(name_target.as_str())?,
                r.is_observable(),
                RegulationsState::monotonicity_from_lib_pbn(r.get_monotonicity()),
            )?;
        }
        Ok(reg_state)
    }

    /// **(internal)** Static utility method to convert monotonicity from the type used in
    /// lib_param_bn into the type used here.
    fn monotonicity_from_lib_pbn(
        monotonicity: Option<Lib_Pbn_Monotonicity>,
    ) -> Option<Monotonicity> {
        match monotonicity {
            Some(m) => match m {
                Lib_Pbn_Monotonicity::Activation => Some(Monotonicity::Activation),
                Lib_Pbn_Monotonicity::Inhibition => Some(Monotonicity::Inhibition),
            },
            None => None,
        }
    }

    /// **(internal)** Static utility method to convert monotonicity from the type used here
    /// into the type used in lib_param_bn.
    fn monotonicity_to_lib_pbn(monotonicity: Option<Monotonicity>) -> Option<Lib_Pbn_Monotonicity> {
        match monotonicity {
            Some(m) => match m {
                Monotonicity::Activation => Some(Lib_Pbn_Monotonicity::Activation),
                Monotonicity::Inhibition => Some(Lib_Pbn_Monotonicity::Inhibition),
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::{RegulationsState, VarId};
    use biodivine_lib_param_bn::RegulatoryGraph;

    #[test]
    fn test_to_reg_graph() {
        let mut reg_state = RegulationsState::new();
        let var_id_a = VarId::new("a").unwrap();
        let var_id_b = VarId::new("b").unwrap();
        reg_state.add_var(var_id_a, "a").unwrap();
        reg_state.add_var(var_id_b, "b").unwrap();
        reg_state.add_regulation_by_str("a -> b").unwrap();
        reg_state.add_regulation_by_str("b -> a").unwrap();

        let reg_graph = reg_state.to_reg_graph();

        assert_eq!(reg_graph.num_vars(), 2);
        assert_eq!(
            reg_graph.regulators(reg_graph.find_variable("a").unwrap()),
            vec![reg_graph.find_variable("b").unwrap()]
        );

        let reg_state_back = RegulationsState::from_reg_graph(reg_graph).unwrap();
        assert_eq!(reg_state, reg_state_back);
    }

    #[test]
    fn test_from_reg_graph() {
        let mut reg_graph = RegulatoryGraph::new(vec!["a".to_string(), "b".to_string()]);
        reg_graph.add_string_regulation("a -> b").unwrap();
        reg_graph.add_string_regulation("b -> a").unwrap();

        let reg_state = RegulationsState::from_reg_graph(reg_graph.clone()).unwrap();
        assert_eq!(reg_state.num_vars(), 2);

        let reg_graph_back = reg_state.to_reg_graph();
        assert_eq!(reg_graph, reg_graph_back);
    }
}
