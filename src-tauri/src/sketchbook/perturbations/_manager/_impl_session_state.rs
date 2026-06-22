use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::{ChangeIdData, PerturbationData};
use crate::sketchbook::event_utils::{
    make_refresh_event, make_reversible, mk_perturb_event, mk_perturb_state_change,
};
use crate::sketchbook::ids::{PerturbationId, VarId};
use crate::sketchbook::perturbations::{Perturbation, PerturbationManager};
use crate::sketchbook::JsonSerde;

/* Constants for event path segments for various events. */

// Add a new prepared perturbation
const ADD_PATH: &str = "add";
// Add a default variant of a perturbation
const ADD_DEFAULT_PATH: &str = "add_default";
// Remove a perturbation
const REMOVE_PATH: &str = "remove";
// Set ID of a perturbation
const SET_ID_PATH: &str = "set_id";
// Change variable ID in all perturbations referencing that variable
const SET_VAR_ID_EVERYWHERE_PATH: &str = "set_var_id_everywhere";
// Set content of a perturbation
const SET_CONTENT_PATH: &str = "set_content";
// Refresh all perturbations
const GET_ALL_PERTURBATIONS_PATH: &str = "get_all_perturbations";

impl SessionHelper for PerturbationManager {}

impl SessionState for PerturbationManager {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        let component_name = "perturbations";

        // When adding new perturbation, the `at_path` starts with "add" (or "add_default")
        // When editing existing perturbations, the `at_path` continues with "perturbation_id" and "action"

        if Self::starts_with(ADD_DEFAULT_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_default_perturb(event)
        } else if Self::starts_with(ADD_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_perturb(event)
        } else if Self::starts_with(SET_VAR_ID_EVERYWHERE_PATH, at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            // Get the payload - json string encoding the ID change data
            let payload = Self::clone_payload_str(event, component_name)?;
            let change_id_data = ChangeIdData::from_json_str(&payload)?;
            let old_var_id = VarId::new(&change_id_data.original_id)?;
            let new_var_id = VarId::new(&change_id_data.new_id)?;

            // Change values of all perturbations that reference this variable (ignoring the rest)
            for (_, perturb) in self.perturbations.iter_mut() {
                perturb.set_var_id_if_present(&old_var_id, new_var_id.clone());
            }

            // The state change is just a list of all perturbations
            let mut perturbations_list: Vec<PerturbationData> = self
                .perturbations
                .iter()
                .map(|(id, perturb)| PerturbationData::from_perturbation(id, perturb))
                .collect();
            perturbations_list.sort_by(|a, b| a.id.cmp(&b.id));
            let state_change = Event {
                path: vec![
                    "sketch".to_string(),
                    "perturbations".to_string(),
                    "all_perturb_updated".to_string(),
                ],
                payload: Some(serde_json::to_string(&perturbations_list)?),
            };

            // Prepare the reverse event (setting the original ID back)
            let reverse_id_change_data =
                ChangeIdData::new(&change_id_data.new_id, &change_id_data.original_id);
            let payload = reverse_id_change_data.to_json_str();
            let reverse_event = mk_perturb_event(&[SET_VAR_ID_EVERYWHERE_PATH], Some(&payload));

            Ok(make_reversible(state_change, event, reverse_event))
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let perturb_id_str = at_path.first().unwrap();
            let perturb_id = self.get_perturbation_id(perturb_id_str)?;
            self.event_modify_perturb(event, &at_path[1..], perturb_id)
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        let component_name = "perturbations";

        // Currently three options: get all datasets, a single dataset, a single observation
        match at_path.first() {
            Some(&GET_ALL_PERTURBATIONS_PATH) => {
                Self::assert_path_length(at_path, 1, component_name)?;
                let mut perturbations_list: Vec<PerturbationData> = self
                    .perturbations
                    .iter()
                    .map(|(id, pert)| PerturbationData::from_perturbation(id, pert))
                    .collect();
                // Return the list sorted, so that it is deterministic
                perturbations_list.sort_by(|a, b| a.id.cmp(&b.id));
                make_refresh_event(full_path, perturbations_list)
            }
            _ => Self::invalid_path_error_generic(at_path),
        }
    }
}

/// Implementation for events related to modifying perturbations.
impl PerturbationManager {
    /// Perform event of adding a new `perturbation` to this `PerturbationManager`.
    pub(super) fn event_add_perturb(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "perturbations";

        // Get payload components and perform the event
        let payload = Self::clone_payload_str(event, component_name)?;
        let pert_data = PerturbationData::from_json_str(payload.as_str())?;
        let perturbation = pert_data.to_perturbation()?;
        self.add_perturbation_by_str(&pert_data.id, perturbation)?;

        // Prepare the state-change and reverse event (which is a remove event)
        let reverse_event = mk_perturb_event(&[&pert_data.id, "remove"], None);
        Ok(make_reversible(event.clone(), event, reverse_event))
    }

    /// Perform event of adding a new DEFAULT `perturbation` of given variant
    /// to this `PerturbationManager`.
    pub(super) fn event_add_default_perturb(
        &mut self,
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "perturbations";

        Self::assert_payload_empty(event, component_name)?;

        // Start indexing at 1
        let perturb_id = self.generate_perturbation_id("perturb", Some(1));
        let perturbation = Perturbation::new_empty(perturb_id.as_str());
        let pert_data = PerturbationData::from_perturbation(&perturb_id, &perturbation);

        // Actually add the perturbation
        self.add_perturbation_by_str(&pert_data.id, perturbation)?;

        // Prepare the state-change (which is add event) and reverse event (which is a remove event)
        let state_change = mk_perturb_state_change(&["add"], &pert_data);
        let reverse_event = mk_perturb_event(&[&pert_data.id, "remove"], None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of modifying or removing existing `perturbation` of this
    /// `PerturbationManager`.
    pub(super) fn event_modify_perturb(
        &mut self,
        event: &Event,
        at_path: &[&str],
        perturb_id: PerturbationId,
    ) -> Result<Consumed, DynError> {
        let component_name = "perturbations";

        if Self::starts_with(REMOVE_PATH, at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // Save the original perturbation data for state change and reverse event
            let original_perturb = self.get_perturbation(&perturb_id)?.clone();
            let pert_data = PerturbationData::from_perturbation(&perturb_id, &original_perturb);

            // Perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_perturbation(&perturb_id)?;
            let state_change = mk_perturb_state_change(&["remove"], &pert_data);

            // Prepare the reverse 'add' event (path has no ids, all info carried by payload)
            let payload = pert_data.to_json_str();
            let reverse_event = mk_perturb_event(&["add"], Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_ID_PATH, at_path).is_some() {
            // Get the payload - string for "new_id"
            let new_id = Self::clone_payload_str(event, component_name)?;
            if perturb_id.as_str() == new_id.as_str() {
                return Ok(Consumed::NoChange);
            }

            // Perform the event, prepare the state-change variant (move id from path to payload)
            self.set_perturbation_id_by_str(perturb_id.as_str(), new_id.as_str())?;
            let id_change_data = ChangeIdData::new(perturb_id.as_str(), new_id.as_str());
            let state_change = mk_perturb_state_change(&["set_id"], &id_change_data);

            // Prepare the reverse event (setting the original ID back)
            let payload = perturb_id.as_str();
            let reverse_event = mk_perturb_event(&[new_id.as_str(), "set_id"], Some(payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with(SET_CONTENT_PATH, at_path).is_some() {
            // Get the payload - json string encoding a new perturbation data
            let payload = Self::clone_payload_str(event, component_name)?;
            let new_perturb_data = PerturbationData::from_json_str(&payload)?;
            let new_perturb = new_perturb_data.to_perturbation()?;
            let orig_perturb = self.get_perturbation(&perturb_id)?;
            if orig_perturb == &new_perturb {
                return Ok(Consumed::NoChange);
            }

            // Perform the event, prepare the state-change variant (move id from path to payload)
            let orig_pert_data = PerturbationData::from_perturbation(&perturb_id, orig_perturb);
            self.swap_perturbation_content(&perturb_id, new_perturb)?;
            let state_change = mk_perturb_state_change(&["set_content"], &new_perturb_data);

            // Prepare the reverse event (setting the original ID back)
            let reverse_at_path = [perturb_id.as_str(), "set_content"];
            let payload = orig_pert_data.to_json_str();
            let reverse_event = mk_perturb_event(&reverse_at_path, Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }
}
