use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::DynError;
use crate::sketchbook::data_structs::RegulationData;
use crate::sketchbook::event_utils::{make_reversible, mk_model_event, mk_model_state_change};
use crate::sketchbook::ids::VarId;
use crate::sketchbook::model::{Essentiality, ModelState, Monotonicity};
use crate::sketchbook::JsonSerde;

/// Implementation for events related to `regulations` of the model.
impl ModelState {
    /// Perform event of adding a new `regulation` component to this `ModelState`.
    pub(super) fn event_add_regulation(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        // get payload components (json for RegulationData containing "regulator", "target", "sign", "essential")
        let payload = Self::clone_payload_str(event, component_name)?;
        let regulation_data = RegulationData::from_json_str(payload.as_str())?;
        let regulator_id = self.get_var_id(&regulation_data.regulator)?;
        let target_id = self.get_var_id(&regulation_data.target)?;
        let sign: Monotonicity = regulation_data.sign;
        let essential: Essentiality = regulation_data.essential;

        // perform the event
        self.add_regulation(regulator_id, target_id, essential, sign)?;

        // prepare the state-change and reverse event (which is a remove event)
        let reverse_at_path = [
            "regulation",
            &regulation_data.regulator,
            &regulation_data.target,
            "remove",
        ];
        let reverse_event = mk_model_event(&reverse_at_path, None);
        Ok(make_reversible(event.clone(), event, reverse_event))
    }

    /// Perform event of modifying or removing existing `regulation` component of this `ModelState`.
    pub(super) fn event_modify_regulation(
        &mut self,
        event: &Event,
        at_path: &[&str],
        regulator_id: VarId,
        target_id: VarId,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        if Self::starts_with("remove", at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // save the original regulation data for state change and reverse event
            let original_reg = self.get_regulation(&regulator_id, &target_id)?.clone();
            let reg_data = RegulationData::from_reg(&original_reg);

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_regulation(&regulator_id, &target_id)?;
            let state_change = mk_model_state_change(&["regulation", "remove"], &reg_data);

            // prepare the reverse 'add' event (path has no ids, all info carried by payload)
            let reverse_at_path = ["regulation", "add"];
            let payload = reg_data.to_json_str();
            let reverse_event = mk_model_event(&reverse_at_path, Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with("set_sign", at_path).is_some() {
            // get the payload - a string for the "new_sign"
            let sign_str = Self::clone_payload_str(event, component_name)?;
            let new_sign = Monotonicity::from_json_str(&sign_str)?;

            let original_reg = self.get_regulation(&regulator_id, &target_id)?;
            let orig_sign = *original_reg.get_sign();

            if orig_sign == new_sign {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.change_regulation_sign(&regulator_id, &target_id, &new_sign)?;
            let new_reg = self.get_regulation(&regulator_id, &target_id)?;
            let reg_data = RegulationData::from_reg(new_reg);
            let state_change = mk_model_state_change(&["regulation", "set_sign"], &reg_data);

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(orig_sign.to_json_str());
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with("set_essentiality", at_path).is_some() {
            // get the payload - a string for the "new_essentiality"
            let essentiality_str = Self::clone_payload_str(event, component_name)?;
            let new_essentiality = Essentiality::from_json_str(&essentiality_str)?;
            let original_reg = self.get_regulation(&regulator_id, &target_id)?;
            let orig_essentiality = *original_reg.get_essentiality();
            if orig_essentiality == new_essentiality {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.change_regulation_essentiality(&regulator_id, &target_id, &new_essentiality)?;
            let new_reg = self.get_regulation(&regulator_id, &target_id)?;
            let reg_data = RegulationData::from_reg(new_reg);
            let state_change =
                mk_model_state_change(&["regulation", "set_essentiality"], &reg_data);

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(orig_essentiality.to_json_str());
            Ok(make_reversible(state_change, event, reverse_event))
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }

    /// Perform events related to `regulations` component of this `ModelState`.
    pub(super) fn perform_regulation_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        // there is either adding of a new regulation, or editing/removing of an existing one
        // when adding new regulation, the `at_path` is just ["add"]
        // when editing existing variable, the `at_path` is ["regulator", "target", "<action>"]

        if Self::starts_with("add", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_regulation(event)
        } else {
            Self::assert_path_length(at_path, 3, component_name)?;
            let regulator_id_str = at_path.first().unwrap();
            let target_id_str = at_path.get(1).unwrap();
            let regulator_id = self.get_var_id(regulator_id_str)?;
            let target_id = self.get_var_id(target_id_str)?;

            self.event_modify_regulation(event, &at_path[2..], regulator_id, target_id)
        }
    }
}
