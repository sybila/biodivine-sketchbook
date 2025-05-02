use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::DynError;
use crate::sketchbook::data_structs::{RegulationData, StatPropertyData};
use crate::sketchbook::event_utils::{
    make_reversible, mk_model_event, mk_model_state_change, mk_stat_prop_event,
};
use crate::sketchbook::ids::VarId;
use crate::sketchbook::model::{Essentiality, ModelState, Monotonicity};
use crate::sketchbook::properties::shortcuts::*;
use crate::sketchbook::properties::StatProperty;
use crate::sketchbook::JsonSerde;

/* Constants for event path segments in `ModelState` related to regulations. */

// add new regulation, and also propagate changes into static properties
const ADD_REGULATION_PATH: &str = "add";
// add new regulation (without additional changes)
const ADD_RAW_REGULATION_PATH: &str = "add_raw";
// remove regulation, and also propagate changes into static properties
const REMOVE_REGULATION_PATH: &str = "remove";
// remove regulation (without additional changes)
const REMOVE_RAW_REGULATION_PATH: &str = "remove_raw";
// set regulation's sign, and also propagate changes into static properties
const SET_SIGN_PATH: &str = "set_sign";
// set regulation's sign (without additional changes)
const SET_SIGN_RAW_PATH: &str = "set_sign_raw";
// set regulation's essentiality, and also propagate changes into static properties
const SET_ESSENTIALITY_PATH: &str = "set_essentiality";
// set regulation's essentiality (without additional changes)
const SET_ESSENTIALITY_RAW_PATH: &str = "set_essentiality_raw";

/// Implementation for events related to `regulations` of the model.
impl ModelState {
    /// Perform events related to `regulations` component of this `ModelState`.
    pub(super) fn perform_regulation_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        // there is either adding of a new regulation, or editing/removing of an existing one
        // when adding new regulation, the `at_path` is just ["add"] or ["add_raw"]
        // when editing existing variable, the `at_path` is ["regulator", "target", "<action>"]

        if Self::starts_with(ADD_REGULATION_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_regulation(event)
        } else if Self::starts_with(ADD_RAW_REGULATION_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_regulation_raw(event)
        } else {
            Self::assert_path_length(at_path, 3, component_name)?;
            let regulator_id_str = at_path.first().unwrap();
            let target_id_str = at_path.get(1).unwrap();
            let regulator_id = self.get_var_id(regulator_id_str)?;
            let target_id = self.get_var_id(target_id_str)?;

            self.event_modify_regulation(event, &at_path[2..], regulator_id, target_id)
        }
    }

    /// Perform event of adding a new `regulation` component to this `ModelState`, and also
    /// add corresponding static properties.
    ///
    /// This breaks the event down into atomic events - first to create corresponding static
    /// properties, and then to make the regulation itself.
    pub(super) fn event_add_regulation(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        // parse the payload
        let payload = Self::clone_payload_str(event, component_name)?;
        let reg_data = RegulationData::from_json_str(payload.as_str())?;

        let mut event_list = Vec::new();
        let input_var = VarId::new(&reg_data.regulator)?;
        let target_var = VarId::new(&reg_data.target)?;

        // events of adding the corresponding properties for monotonicity/essentiality in case it
        // is not unknown variant
        if reg_data.essential != Essentiality::Unknown {
            let prop_id = StatProperty::get_reg_essentiality_prop_id(&input_var, &target_var);
            let prop = mk_reg_essentiality_prop(&input_var, &target_var, reg_data.essential);
            let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
            let prop_event = mk_stat_prop_event(&["add"], Some(&prop_payload));
            event_list.push(prop_event);
        }
        if reg_data.sign != Monotonicity::Unknown {
            let prop_id = StatProperty::get_reg_monotonicity_prop_id(&input_var, &target_var);
            let prop = mk_reg_monotonicity_prop(&input_var, &target_var, reg_data.sign);
            let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
            let prop_event = mk_stat_prop_event(&["add"], Some(&prop_payload));
            event_list.push(prop_event);
        }

        // and finally, the event of adding the raw regulation itself (the event list will be
        // reversed, this being the first of the events with all the checks)
        let reg_event = mk_model_event(&["regulation", "add_raw"], Some(&payload));
        event_list.push(reg_event);

        Ok(Consumed::Restart(event_list))
    }

    /// Perform event of adding a `regulation` component to this `ModelState`.
    ///
    /// This version is only adding the raw regulation, and not the corresponding static properties.
    /// It is expected that `event_add_regulation` is called first, handling the actual division
    /// into this event + event for adding the properties.
    pub(super) fn event_add_regulation_raw(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/regulation";

        // parse the payload
        let payload = Self::clone_payload_str(event, component_name)?;
        let reg_data = RegulationData::from_json_str(payload.as_str())?;
        let regulator_id = self.get_var_id(&reg_data.regulator)?;
        let target_id = self.get_var_id(&reg_data.target)?;
        let sign: Monotonicity = reg_data.sign;
        let essential: Essentiality = reg_data.essential;

        // perform the event
        self.add_regulation(regulator_id, target_id, essential, sign)?;

        // prepare the state-change and reverse event (which is a remove event)
        let state_change = mk_model_state_change(&["regulation", "add"], &reg_data);
        let reverse_at_path = [
            "regulation",
            &reg_data.regulator,
            &reg_data.target,
            "remove_raw",
        ];
        let reverse_event = mk_model_event(&reverse_at_path, None);
        Ok(make_reversible(state_change, event, reverse_event))
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

        if Self::starts_with(REMOVE_REGULATION_PATH, at_path).is_some() {
            // To remove a regulation, all its essentiality/monotonicity properties must also be removed.
            // We break this event down into atomic sub-events to ensure that we can undo this operation
            // later. We prepare a set of events to remove all the properties, and then remove the
            // regulation atomically (all as separate undo-able events).
            let mut event_list = Vec::new();

            // the event of removing the raw regulation itself
            // the event list will be reversed, and this will become the last of the sub-events processed
            let reg_event_path = [
                "regulation",
                regulator_id.as_str(),
                target_id.as_str(),
                "remove_raw",
            ];
            let reg_event = mk_model_event(&reg_event_path, None);
            event_list.push(reg_event);

            // events of removing the corresponding properties for monotonicity/essentiality in
            // case it is not unknown variant
            let original_reg = self.get_regulation(&regulator_id, &target_id)?.clone();
            if *original_reg.get_essentiality() != Essentiality::Unknown {
                // there is at max one essentiality property for a regulation
                let prop_id = StatProperty::get_reg_essentiality_prop_id(&regulator_id, &target_id);
                let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                event_list.push(prop_event);
            }
            if *original_reg.get_sign() != Monotonicity::Unknown {
                // there is at max one monotonicity property for a regulation
                let prop_id = StatProperty::get_reg_monotonicity_prop_id(&regulator_id, &target_id);
                let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                event_list.push(prop_event);
            }
            Ok(Consumed::Restart(event_list))
        } else if Self::starts_with(REMOVE_RAW_REGULATION_PATH, at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // save the original regulation data for state change and reverse event
            let original_reg = self.get_regulation(&regulator_id, &target_id)?.clone();
            let reg_data = RegulationData::from_reg(&original_reg);

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_regulation(&regulator_id, &target_id)?;
            let state_change = mk_model_state_change(&["regulation", "remove"], &reg_data);

            // prepare the reverse 'add' event (path has no ids, all info carried by payload)
            let reverse_at_path = ["regulation", "add_raw"];
            let payload = reg_data.to_json_str();
            let reverse_event = mk_model_event(&reverse_at_path, Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_SIGN_PATH, at_path).is_some() {
            // This event is broken down into atomic events - first to potentially create corresponding
            // static property, and then to modify the regulation itself.

            // get the payload - a string for the "new_sign"
            let sign_str = Self::clone_payload_str(event, component_name)?;
            let new_sign = Monotonicity::from_json_str(&sign_str)?;

            let original_reg = self.get_regulation(&regulator_id, &target_id)?;
            let orig_sign = *original_reg.get_sign();

            if orig_sign == new_sign {
                return Ok(Consumed::NoChange);
            }

            // now we must handle the event itself, and all potential static property changes
            let mut event_list = Vec::new();
            // the event of changing the raw regulation sign (payload stays the same)
            let reg_event_path = [
                "regulation",
                regulator_id.as_str(),
                target_id.as_str(),
                "set_sign_raw",
            ];
            let reg_event = mk_model_event(&reg_event_path, Some(&sign_str));
            event_list.push(reg_event);

            // now the events of modifying/adding/removing corresponding static property
            // note we have already checked that `orig_sign` and `new_sign` are different
            let prop_id = StatProperty::get_reg_monotonicity_prop_id(&regulator_id, &target_id);
            if orig_sign == Monotonicity::Unknown {
                // before there was no static prop, now we have to add it
                let prop = mk_reg_monotonicity_prop(&regulator_id, &target_id, new_sign);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event = mk_stat_prop_event(&["add"], Some(&prop_payload));
                event_list.push(prop_event);
            } else if new_sign == Monotonicity::Unknown {
                // before there was a static prop, now we have to remove it
                let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                event_list.push(prop_event);
            } else {
                // there is a static prop, and we must change its sign
                let prop = mk_reg_monotonicity_prop(&regulator_id, &target_id, new_sign);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event =
                    mk_stat_prop_event(&[prop_id.as_str(), "set_content"], Some(&prop_payload));
                event_list.push(prop_event);
            }

            Ok(Consumed::Restart(event_list))
        } else if Self::starts_with(SET_SIGN_RAW_PATH, at_path).is_some() {
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
        } else if Self::starts_with(SET_ESSENTIALITY_PATH, at_path).is_some() {
            // This event is broken down into atomic events - first to potentially create corresponding
            // static property, and then to modify the regulation itself.

            // get the payload - a string for the "new_essentiality"
            let essentiality_str = Self::clone_payload_str(event, component_name)?;
            let new_essentiality = Essentiality::from_json_str(&essentiality_str)?;
            let original_reg = self.get_regulation(&regulator_id, &target_id)?;
            let orig_essentiality = *original_reg.get_essentiality();
            if orig_essentiality == new_essentiality {
                return Ok(Consumed::NoChange);
            }

            // now we must handle the event itself, and all potential static property changes
            let mut event_list = Vec::new();
            // the event of changing the raw regulation essentiality (payload stays the same)
            let reg_event_path = [
                "regulation",
                regulator_id.as_str(),
                target_id.as_str(),
                "set_essentiality_raw",
            ];
            let reg_event = mk_model_event(&reg_event_path, Some(&essentiality_str));
            event_list.push(reg_event);

            // now the events of modifying/adding/removing corresponding static property
            // note we have already checked that `orig_essentiality` and `new_essentiality` are different
            let prop_id = StatProperty::get_reg_essentiality_prop_id(&regulator_id, &target_id);
            if orig_essentiality == Essentiality::Unknown {
                // before there was no static prop, now we have to add it
                let prop = mk_reg_essentiality_prop(&regulator_id, &target_id, new_essentiality);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event = mk_stat_prop_event(&["add"], Some(&prop_payload));
                event_list.push(prop_event);
            } else if new_essentiality == Essentiality::Unknown {
                // before there was a static prop, now we have to remove it
                let prop_event = mk_stat_prop_event(&[prop_id.as_str(), "remove"], None);
                event_list.push(prop_event);
            } else {
                // there is a static prop, and we must change its essentiality
                let prop = mk_reg_essentiality_prop(&regulator_id, &target_id, new_essentiality);
                let prop_payload = StatPropertyData::from_property(&prop_id, &prop).to_json_str();
                let prop_event =
                    mk_stat_prop_event(&[prop_id.as_str(), "set_content"], Some(&prop_payload));
                event_list.push(prop_event);
            }

            Ok(Consumed::Restart(event_list))
        } else if Self::starts_with(SET_ESSENTIALITY_RAW_PATH, at_path).is_some() {
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
}
