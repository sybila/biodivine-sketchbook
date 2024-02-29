use crate::app::event::Event;
use crate::app::state::SessionHelper;
use crate::app::DynError;
use crate::sketchbook::data_structs::{
    LayoutData, LayoutNodeData, RegulationData, UninterpretedFnData, UpdateFnData, VariableData,
};
use crate::sketchbook::ModelState;

/// Implementation for `refresh` (getter) events.
impl ModelState {
    /// Get a list of all variables.
    pub(super) fn refresh_variables(&self, full_path: &[String]) -> Result<Event, DynError> {
        let variable_list: Vec<VariableData> = self
            .variables
            .iter()
            .map(|(id, data)| VariableData::from_var(id, data))
            .collect();

        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(serde_json::to_string(&variable_list)?),
        })
    }

    /// Get a list of all uninterpreted fns.
    pub(super) fn refresh_uninterpreted_fns(
        &self,
        full_path: &[String],
    ) -> Result<Event, DynError> {
        let uninterpreted_fn_list: Vec<UninterpretedFnData> = self
            .uninterpreted_fns
            .iter()
            .map(|(id, data)| UninterpretedFnData::from_uninterpreted_fn(id, data))
            .collect();

        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(serde_json::to_string(&uninterpreted_fn_list)?),
        })
    }

    /// Get a list of all regulations.
    pub(super) fn refresh_regulations(&self, full_path: &[String]) -> Result<Event, DynError> {
        let regulation_list: Vec<RegulationData> = self
            .regulations
            .iter()
            .map(RegulationData::from_reg)
            .collect();

        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(serde_json::to_string(&regulation_list)?),
        })
    }

    /// Get a list of all layouts (just basic information like IDs and names).
    pub(super) fn refresh_layouts(&self, full_path: &[String]) -> Result<Event, DynError> {
        let layout_list: Vec<LayoutData> = self
            .layouts
            .iter()
            .map(|(id, layout)| LayoutData::from_layout(id, layout))
            .collect();

        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(serde_json::to_string(&layout_list)?),
        })
    }

    /// Get a list with all nodes in a specified layout.
    pub(super) fn refresh_layout_nodes(
        &self,
        full_path: &[String],
        at_path: &[&str],
    ) -> Result<Event, DynError> {
        Self::assert_path_length(at_path, 1, "model/layout_nodes")?;
        let layout_id_str = at_path.first().unwrap();
        let layout_id = self.get_layout_id(layout_id_str)?;
        let layout = self.get_layout(&layout_id)?;

        let node_list: Vec<LayoutNodeData> = layout
            .layout_nodes()
            .map(|(var_id, node)| LayoutNodeData::from_node(&layout_id, var_id, node))
            .collect();

        // remove the id from the path
        let mut result_path = full_path.to_vec();
        result_path.pop();

        Ok(Event {
            path: result_path,
            payload: Some(serde_json::to_string(&node_list)?),
        })
    }

    /// Get a list of all update functions (with corresponding var id).
    pub(super) fn refresh_update_fns(&self, full_path: &[String]) -> Result<Event, DynError> {
        let update_fns_list: Vec<UpdateFnData> = self
            .update_fns
            .iter()
            .map(|(id, update_fn)| UpdateFnData::from_update_fn(id, update_fn))
            .collect();

        Ok(Event {
            path: full_path.to_vec(),
            payload: Some(serde_json::to_string(&update_fns_list)?),
        })
    }
}
