use crate::app::event::Event;
use crate::app::state::Consumed;
use crate::app::{AeonError, DynError};
use crate::sketchbook::data_structs::ObservationData;
use crate::sketchbook::event_utils::{make_reversible, make_state_change};
use crate::sketchbook::observations::Dataset;
use crate::sketchbook::{DatasetId, ObservationId};
use std::str::FromStr;

/// Implementation for events related to modifying `Datasets` by adding/removing `observations`.
/// `Dataset` does not implement `SessionState` trait directly. Instead, it just offers methods
/// to perform certain events, after the preprocessing is done by `ObservationManager`.
impl Dataset {
    /// Perform event of adding a new `observation` to this `Dataset`.
    pub(in crate::sketchbook::observations) fn event_add_observation(
        &mut self,
        event: &Event,
        dataset_id: DatasetId,
    ) -> Result<Consumed, DynError> {
        let component_name = "observation_manager/dataset";

        // get payload components and perform the event
        let payload = event.payload.clone().ok_or(format!(
            "This event to `{component_name}` cannot carry empty payload."
        ))?;
        let observation_data = ObservationData::from_str(payload.as_str())?;
        let observation = observation_data.to_observation()?;
        self.push_observation(observation)?;

        // prepare the state-change variant (remove IDs from the path)
        let state_change_path = ["observation_manager", "dataset", "add"];
        let state_change = make_state_change(&state_change_path, &observation_data);
        // prepare the reverse event (which is a remove event)
        let reverse_path = [
            "observation_manager",
            &dataset_id.as_str(),
            &observation_data.id,
            "remove",
        ];
        let reverse_event = Event::build(&reverse_path, None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of modifying or removing existing `observation` component of this `Dataset`.
    pub(in crate::sketchbook::observations) fn event_modify_observation(
        &mut self,
        event: &Event,
        action: &str,
        dataset_id: DatasetId,
        obs_id: ObservationId,
    ) -> Result<Consumed, DynError> {
        let component_name = "observation_manager/dataset";

        if action == "remove" {
            // save the original observation data for state change and reverse event
            let original_obs = self.get_observation(&obs_id)?.clone();
            let obs_data = ObservationData::from_obs(&original_obs, &dataset_id);

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_observation(&obs_id)?;
            let state_change_path = ["observation_manager", "dataset", "remove"];
            let state_change = make_state_change(&state_change_path, &obs_data);

            // prepare the reverse 'add' event (path has no ids, all info carried by payload)
            let reverse_path = ["observation_manager", dataset_id.as_str(), "add"];
            let reverse_event = Event::build(&reverse_path, Some(&obs_data.to_string()));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if action == "set_id" {
            // todo: change observation's id
            todo!()
        } else if action == "change_data" {
            // todo: change observation's whole value list to a new list
            todo!()
        } else {
            AeonError::throw(format!(
                "`{component_name}` cannot perform action `{action}`."
            ))
        }
    }
}
