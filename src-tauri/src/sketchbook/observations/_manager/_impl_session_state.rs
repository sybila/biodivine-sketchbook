use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::{ChangeIdData, DatasetData, ObservationData};
use crate::sketchbook::event_utils::{make_refresh_event, make_reversible, make_state_change};
use crate::sketchbook::observations::ObservationManager;
use crate::sketchbook::{DatasetId, ObservationId};
use std::str::FromStr;

impl SessionHelper for ObservationManager {}

impl SessionState for ObservationManager {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        let component_name = "observation_manager";

        // there is either adding of a new dataset, or modifying/removing an existing one
        // when adding new dataset, the `at_path` is just ["add"]
        // when editing existing dataset, the `at_path` is ["dataset_id", ...]

        if Self::starts_with("add", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_dataset(event)
        } else {
            let dataset_id_str = at_path.first().unwrap();
            let dataset_id = self.get_dataset_id(dataset_id_str)?;
            self.event_modify_dataset(event, &at_path[1..], dataset_id)
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        let component_name = "observation_manager";

        // currently three options: get all datasets, a single dataset, a single observation
        match at_path.first() {
            Some(&"get_all_datasets") => {
                Self::assert_path_length(at_path, 0, component_name)?;
                let dataset_list: Vec<DatasetData> = self
                    .datasets
                    .iter()
                    .map(|(id, dataset)| DatasetData::from_dataset(dataset, id))
                    .collect();
                make_refresh_event(full_path, dataset_list)
            }
            Some(&"get_dataset") => {
                // path specifies dataset's ID
                Self::assert_path_length(at_path, 1, component_name)?;
                let dataset_id_str = at_path[0];

                let dataset_id = self.get_dataset_id(dataset_id_str)?;
                let dataset = self.get_dataset(&dataset_id)?;
                let dataset_data = DatasetData::from_dataset(dataset, &dataset_id);
                let payload = Some(dataset_data.to_string());

                let mut path = full_path.to_vec();
                path.pop(); // remove the id from the path

                Ok(Event { path, payload })
            }
            Some(&"get_observation") => {
                // path specifies 1) dataset's ID and 2) observation's ID
                Self::assert_path_length(at_path, 2, component_name)?;
                let dataset_id_str = at_path[0];
                let dataset_id = self.get_dataset_id(dataset_id_str)?;
                let obs_id_str = at_path[1];

                let observation = self.get_observation_by_str(dataset_id_str, obs_id_str)?;
                let obs_data = ObservationData::from_obs(observation, &dataset_id);
                let payload = Some(obs_data.to_string());

                let mut path = full_path.to_vec();
                path.truncate(path.len() - 2); // remove the two ids from the path

                Ok(Event { path, payload })
            }
            _ => Self::invalid_path_error_generic(at_path),
        }
    }
}

/// Implementation for events related to modifying `datasets`.
impl ObservationManager {
    /// Perform event of adding a new `dataset` to this `ObservationManager`.
    pub(super) fn event_add_dataset(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "observation_manager";

        // get payload components and perform the event
        let payload = Self::clone_payload_str(event, component_name)?;
        let dataset_data = DatasetData::from_str(payload.as_str())?;
        let dataset = dataset_data.to_dataset()?;
        self.add_dataset_by_str(&dataset_data.id, dataset)?;

        // prepare the state-change and reverse event (which is a remove event)
        let reverse_path = ["observation_manager", &dataset_data.id, "remove"];
        let reverse_event = Event::build(&reverse_path, None);
        Ok(make_reversible(event.clone(), event, reverse_event))
    }

    /// Perform event of modifying or removing existing `dataset` component of this
    /// `ObservationManager`.
    pub(super) fn event_modify_dataset(
        &mut self,
        event: &Event,
        at_path: &[&str],
        dataset_id: DatasetId,
    ) -> Result<Consumed, DynError> {
        let component_name = "observation_manager";

        // there are two possible options:
        //     1) modify a dataset directly, with `at_path` being just [<ACTION>]
        //     2) modify a specific observation with `at_path` being ["observation_id", ...]
        // the second option is handled by the dataset instance itself

        if Self::starts_with("remove", at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // save the original dataset data for state change and reverse event
            let original_dataset = self.get_dataset(&dataset_id)?.clone();
            let dataset_data = DatasetData::from_dataset(&original_dataset, &dataset_id);

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_dataset(&dataset_id)?;
            let state_change = make_state_change(&["observation_manager", "remove"], &dataset_data);

            // prepare the reverse 'add' event (path has no ids, all info carried by payload)
            let reverse_path = ["observation_manager", "add"];
            let reverse_event = Event::build(&reverse_path, Some(&dataset_data.to_string()));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with("set_id", at_path).is_some() {
            // get the payload - string for "new_id"
            let new_id = Self::clone_payload_str(event, component_name)?;
            if dataset_id.as_str() == new_id.as_str() {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.set_dataset_id_by_str(dataset_id.as_str(), new_id.as_str())?;
            let id_change_data = ChangeIdData::new(dataset_id.as_str(), new_id.as_str());
            let state_change =
                make_state_change(&["observation_manager", "set_id"], &id_change_data);

            // prepare the reverse event (setting the original ID back)
            let reverse_event_path = ["observation_manager", new_id.as_str(), "set_id"];
            let reverse_event = Event::build(&reverse_event_path, Some(dataset_id.as_str()));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with("change_data", at_path).is_some() {
            // todo: change dataset's whole observation list to a new list
            todo!()
        } else if Self::starts_with("set_var_id", at_path).is_some() {
            // todo: change variable's ID in a dataset
            todo!()
        } else if Self::starts_with("add", at_path).is_some() {
            // Adding observation to a particular dataset is handled by the `Dataset` itself

            // the ID is valid (checked before), we can unwrap
            let dataset = self.datasets.get_mut(&dataset_id).unwrap();
            dataset.event_add_observation(event, dataset_id)
        } else {
            // Finally, this must be a modification of a particular observation
            // The `at_path` must be ["observation_id", <ACTION>]
            // We just extract the particular ID and let the `Dataset` handle it itself
            Self::assert_path_length(at_path, 2, component_name)?;
            let observation_id_str = at_path[0];
            let obs_id = ObservationId::new(observation_id_str)?;
            let action = at_path[1];

            // the ID is valid (checked before), we can unwrap
            let dataset = self.datasets.get_mut(&dataset_id).unwrap();
            dataset.event_modify_observation(event, action, dataset_id, obs_id)
        }
    }
}
