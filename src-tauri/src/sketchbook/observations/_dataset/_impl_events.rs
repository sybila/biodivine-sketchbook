use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::{AeonError, DynError};
use crate::sketchbook::data_structs::{ChangeIdData, ObservationData};
use crate::sketchbook::event_utils::{make_reversible, mk_obs_event, mk_obs_state_change};
use crate::sketchbook::ids::{DatasetId, ObservationId};
use crate::sketchbook::observations::{Dataset, Observation};
use crate::sketchbook::JsonSerde;

/* Constants for event path segments for various events related to `Dataset` observations. */

// remove an observation from the dataset
const REMOVE_OBSERVATION_PATH: &str = "remove";
// set observation's ID
const SET_OBSERVATION_ID_PATH: &str = "set_id";
// set observation's data
const SET_OBSERVATION_DATA_PATH: &str = "set_data";

impl SessionHelper for Dataset {}

/// Implementation for events related to modifying `observations` in a particular `Dataset`.
/// `Dataset` does not implement `SessionState` trait directly. Instead, it just offers methods
/// to perform certain events, after the preprocessing is done by `ObservationManager`.
impl Dataset {
    /// Perform event of adding a new `observation` to the end of this `Dataset`.
    pub(in crate::sketchbook::observations) fn event_push_observation(
        &mut self,
        event: &Event,
        dataset_id: DatasetId,
    ) -> Result<Consumed, DynError> {
        let component_name = "observations/dataset";

        // get payload components and perform the action
        let payload = Self::clone_payload_str(event, component_name)?;
        let observation_data = ObservationData::from_json_str(payload.as_str())?;
        let observation = observation_data.to_observation()?;
        self.push_obs(observation)?;

        // prepare the state-change variant (remove IDs from the path)
        let state_change = mk_obs_state_change(&["push_obs"], &observation_data);
        // prepare the reverse event (which is a pop event)
        let reverse_at_path = [dataset_id.as_str(), "pop_obs"];
        let reverse_event = mk_obs_event(&reverse_at_path, None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of adding a completely new "empty" `observation` to the end of this `Dataset`.
    ///
    /// All its values are `unspecified` and its Id is newly generated.
    pub(in crate::sketchbook::observations) fn event_push_empty_observation(
        &mut self,
        event: &Event,
        dataset_id: DatasetId,
    ) -> Result<Consumed, DynError> {
        // get payload components and perform the action
        // start indexing at 1
        let id = self.generate_obs_id("obs", Some(1));
        let observation = Observation::new_full_unspecified(self.num_variables(), id.as_str())?;
        let observation_data = ObservationData::from_obs(&observation, &dataset_id);
        self.push_obs(observation)?;

        // prepare the state-change variant - classical push_obs event
        let state_change = mk_obs_state_change(&["push_obs"], &observation_data);
        // prepare the reverse event (which is a pop event)
        let reverse_at_path = [dataset_id.as_str(), "pop_obs"];
        let reverse_event = mk_obs_event(&reverse_at_path, None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of removing the last observation from this `Dataset`.
    pub(in crate::sketchbook::observations) fn event_pop_observation(
        &mut self,
        event: &Event,
        dataset_id: DatasetId,
    ) -> Result<Consumed, DynError> {
        if self.num_observations() == 0 {
            return Ok(Consumed::NoChange); // nothing to remove
        }

        // save the original observation data for state change and reverse event
        let last_obs = self.observations.last().unwrap();
        let obs_data = ObservationData::from_obs(last_obs, &dataset_id);

        // perform the action, prepare the state-change variant (move IDs from path to payload)
        self.pop_obs();
        let state_change = mk_obs_state_change(&["pop_obs"], &obs_data);

        // prepare the reverse 'add_last' event (path has no ids, all info carried by payload)
        let reverse_at_path = [dataset_id.as_str(), "push_obs"];
        let payload = obs_data.to_json_str();
        let reverse_event = mk_obs_event(&reverse_at_path, Some(&payload));
        Ok(make_reversible(state_change, event, reverse_event))
    }

    pub(in crate::sketchbook::observations) fn event_modify_observation(
        &mut self,
        event: &Event,
        action: &str,
        dataset_id: DatasetId,
        obs_id: ObservationId,
    ) -> Result<Consumed, DynError> {
        let component_name = "observations/dataset";

        match action {
            REMOVE_OBSERVATION_PATH => {
                // Save the original observation data for state change and reverse event
                let original_obs = self.get_obs(&obs_id)?.clone();
                let obs_data = ObservationData::from_obs(&original_obs, &dataset_id);

                // Perform the action and prepare the state-change variant
                self.remove_obs(&obs_id)?;
                let state_change = mk_obs_state_change(&[REMOVE_OBSERVATION_PATH], &obs_data);

                Ok(Consumed::Irreversible {
                    state_change,
                    reset: true,
                })
            }
            SET_OBSERVATION_ID_PATH => {
                // Get the payload - string for "new_id"
                let new_id = Self::clone_payload_str(event, component_name)?;
                if obs_id.as_str() == new_id.as_str() {
                    return Ok(Consumed::NoChange);
                }

                // Perform the action and prepare the state-change variant
                self.set_obs_id_by_str(obs_id.as_str(), new_id.as_str())?;
                let id_change_data = ChangeIdData::new_with_metadata(
                    obs_id.as_str(),
                    new_id.as_str(),
                    dataset_id.as_str(),
                );
                let state_change = mk_obs_state_change(&[SET_OBSERVATION_ID_PATH], &id_change_data);

                // Prepare the reverse event
                let reverse_at_path = [
                    dataset_id.as_str(),
                    new_id.as_str(),
                    SET_OBSERVATION_ID_PATH,
                ];
                let reverse_event = mk_obs_event(&reverse_at_path, Some(obs_id.as_str()));
                Ok(make_reversible(state_change, event, reverse_event))
            }
            SET_OBSERVATION_DATA_PATH => {
                // Get the payload - string encoding a new modified observation data
                let payload = Self::clone_payload_str(event, component_name)?;
                let new_obs_data = ObservationData::from_json_str(&payload)?;
                let new_obs = new_obs_data.to_observation()?;
                let orig_obs = self.get_obs(&obs_id)?;
                if orig_obs == &new_obs {
                    return Ok(Consumed::NoChange);
                }

                // Perform the action and prepare the state-change variant
                let orig_obs_data = ObservationData::from_obs(orig_obs, &dataset_id);
                self.set_observation_raw(&obs_id, new_obs)?;
                let state_change = mk_obs_state_change(&[SET_OBSERVATION_DATA_PATH], &new_obs_data);

                // Prepare the reverse event
                let reverse_at_path = [
                    dataset_id.as_str(),
                    obs_id.as_str(),
                    SET_OBSERVATION_DATA_PATH,
                ];
                let payload = orig_obs_data.to_json_str();
                let reverse_event = mk_obs_event(&reverse_at_path, Some(&payload));
                Ok(make_reversible(state_change, event, reverse_event))
            }
            _ => AeonError::throw(format!(
                "`{component_name}` cannot perform action `{action}`."
            )),
        }
    }
}
