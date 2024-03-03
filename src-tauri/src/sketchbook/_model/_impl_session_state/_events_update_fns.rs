use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::DynError;
use crate::sketchbook::_model::_impl_session_state::_utils::{make_reversible, make_state_change};
use crate::sketchbook::data_structs::UpdateFnData;
use crate::sketchbook::ModelState;

/// Implementation for events related to `update functions` of the model.
impl ModelState {
    /// Perform events related to `variables` component of this `ModelState`.
    pub(super) fn perform_update_fn_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/update_fn";

        // the path must contain two components - an "id" and "operation"
        Self::assert_path_length(at_path, 2, component_name)?;
        let var_id_str = at_path.first().unwrap();
        let var_id = self.get_var_id(var_id_str)?;

        // update functions cannot be created or removed, only modified
        if Self::starts_with("set_expression", &at_path[1..]).is_some() {
            // get the payload - string for "new_expression"
            let new_expression = Self::clone_payload_str(event, component_name)?;
            let original_expression = self.get_update_fn(&var_id)?.to_string();
            // actually, this check is not that relevant, as the expressions might be "normalized" during parsing
            if new_expression == original_expression {
                return Ok(Consumed::NoChange);
            }

            // perform the event and check (again) that the new parsed version is different than the original
            self.set_update_fn(&var_id, new_expression.as_str())?;
            let fn_data = UpdateFnData::new(var_id.as_str(), self.get_update_fn_string(&var_id)?);
            if fn_data.expression == original_expression {
                return Ok(Consumed::NoChange);
            }

            // prepare state-change and reverse events
            let state_change =
                make_state_change(&["model", "update_fn", "set_expression"], &fn_data);
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(original_expression);
            Ok(make_reversible(state_change, event, reverse_event))
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }
}
