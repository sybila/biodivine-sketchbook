use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::{
    ChangeIdData, DatasetData, DatasetMetaData, ObservationData,
};
use crate::sketchbook::event_utils::{
    make_refresh_event, make_reversible, mk_obs_event, mk_obs_state_change,
};
use crate::sketchbook::ids::{DatasetId, ObservationId};
use crate::sketchbook::observations::{Dataset, ObservationManager};
use crate::sketchbook::JsonSerde;

impl SessionHelper for ObservationManager {}

impl SessionState for ObservationManager {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        let component_name = "observations";

        // there is either adding/loading of a new dataset, or modifying/removing an existing one
        // when adding new dataset, the `at_path` is just ["add"] or ["add_default"]
        // when editing existing dataset, the `at_path` is ["dataset_id", ...]

        if Self::starts_with("add_default", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_default_dataset(event)
        } else if Self::starts_with("add", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_dataset(event)
        } else if Self::starts_with("load", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_load_dataset(event)
        } else {
            let dataset_id_str = at_path.first().unwrap();
            let dataset_id = self.get_dataset_id(dataset_id_str)?;
            self.event_modify_dataset(event, &at_path[1..], dataset_id)
        }
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        let component_name = "observations";

        // currently three options: get all datasets, a single dataset, a single observation
        match at_path.first() {
            Some(&"get_all_datasets") => {
                Self::assert_path_length(at_path, 1, component_name)?;
                let mut dataset_list: Vec<DatasetData> = self
                    .datasets
                    .iter()
                    .map(|(id, dataset)| DatasetData::from_dataset(id, dataset))
                    .collect();
                // return the list sorted, so that it is deterministic
                dataset_list.sort_by(|a, b| a.id.cmp(&b.id));
                make_refresh_event(full_path, dataset_list)
            }
            Some(&"get_dataset") => {
                // path specifies dataset's ID
                Self::assert_path_length(at_path, 2, component_name)?;
                let dataset_id_str = at_path[1];

                let dataset_id = self.get_dataset_id(dataset_id_str)?;
                let dataset = self.get_dataset(&dataset_id)?;
                let dataset_data = DatasetData::from_dataset(&dataset_id, dataset);
                let payload = Some(dataset_data.to_json_str());

                let mut path = full_path.to_vec();
                path.pop(); // remove the id from the path

                Ok(Event { path, payload })
            }
            Some(&"get_observation") => {
                // path specifies 1) dataset's ID and 2) observation's ID
                Self::assert_path_length(at_path, 3, component_name)?;
                let dataset_id_str = at_path[1];
                let dataset_id = self.get_dataset_id(dataset_id_str)?;
                let obs_id_str = at_path[2];

                let observation = self.get_observation_by_str(dataset_id_str, obs_id_str)?;
                let obs_data = ObservationData::from_obs(observation, &dataset_id);
                let payload = Some(obs_data.to_json_str());

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
        let component_name = "observations";

        // get payload components and perform the event
        let payload = Self::clone_payload_str(event, component_name)?;
        let dataset_data = DatasetData::from_json_str(payload.as_str())?;
        let dataset = dataset_data.to_dataset()?;
        self.add_dataset_by_str(&dataset_data.id, dataset)?;

        // prepare the state-change and reverse event (which is a remove event)
        let reverse_event = mk_obs_event(&[&dataset_data.id, "remove"], None);
        Ok(make_reversible(event.clone(), event, reverse_event))
    }

    /// Perform event of adding a new DEFAULT (empty) `dataset` to this `ObservationsManager`.
    pub(super) fn event_add_default_dataset(
        &mut self,
        event: &Event,
    ) -> Result<Consumed, DynError> {
        let component_name = "observations";
        Self::assert_payload_empty(event, component_name)?;

        let dataset = Dataset::default();
        // start indexing at 1
        let dataset_id = self.generate_dataset_id("dataset", Some(1));
        let dataset_data = DatasetData::from_dataset(&dataset_id, &dataset);

        self.add_dataset(dataset_id, dataset)?;

        // prepare the state-change and reverse event (which is a remove event)
        let state_change = mk_obs_state_change(&["add"], &dataset_data);
        let reverse_event = mk_obs_event(&[&dataset_data.id, "remove"], None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of loading (and adding) new `dataset` to this `ObservationManager`.
    pub(super) fn event_load_dataset(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "observations";

        // get the payload - a path to a csv file with dataset
        let file_path = Self::clone_payload_str(event, component_name)?;

        // load the dataset, generate new ID, and add it
        let dataset = Self::load_dataset(&file_path)?;
        // start indexing at 1
        let dataset_id = self.generate_dataset_id("dataset", Some(1));
        let dataset_data = DatasetData::from_dataset(&dataset_id, &dataset);
        self.add_dataset_by_str(&dataset_data.id, dataset)?;

        // prepare the state-change event (which sends the loaded dataset to frontend)
        let state_change = mk_obs_state_change(&["load"], &dataset_data);
        // and also prepare the reverse, which is a classical `remove` event
        let reverse_event = mk_obs_event(&[&dataset_data.id, "remove"], None);
        Ok(make_reversible(state_change, event, reverse_event))
    }

    /// Perform event of modifying or removing existing `dataset` component of this
    /// `ObservationManager`.
    pub(super) fn event_modify_dataset(
        &mut self,
        event: &Event,
        at_path: &[&str],
        dataset_id: DatasetId,
    ) -> Result<Consumed, DynError> {
        let component_name = "observations";

        // there are two possible options:
        //     1) modify a dataset directly, with `at_path` being just [<ACTION>]
        //     2) modify a specific observation with `at_path` being ["observation_id", ...]
        // the second option is handled by the dataset instance itself

        if Self::starts_with("remove", at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            // save the original dataset data for state change and reverse event
            let original_dataset = self.get_dataset(&dataset_id)?.clone();
            let dataset_data = DatasetData::from_dataset(&dataset_id, &original_dataset);

            // perform the event, prepare the state-change variant (move IDs from path to payload)
            self.remove_dataset(&dataset_id)?;
            let state_change = mk_obs_state_change(&["remove"], &dataset_data);

            // prepare the reverse 'add' event (path has no ids, all info carried by payload)
            let payload = dataset_data.to_json_str();
            let reverse_event = mk_obs_event(&["add"], Some(&payload));
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
            let state_change = mk_obs_state_change(&["set_id"], &id_change_data);

            // prepare the reverse event (setting the original ID back)
            let payload = dataset_id.as_str();
            let reverse_event = mk_obs_event(&[new_id.as_str(), "set_id"], Some(payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with("set_content", at_path).is_some() {
            // get the payload - json string encoding a new dataset data
            let payload = Self::clone_payload_str(event, component_name)?;
            let new_dataset_data = DatasetData::from_json_str(&payload)?;
            let new_dataset = new_dataset_data.to_dataset()?;
            let orig_dataset = self.get_dataset(&dataset_id)?;
            if orig_dataset == &new_dataset {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            let orig_dataset_data = DatasetData::from_dataset(&dataset_id, orig_dataset);
            self.swap_dataset_content(&dataset_id, new_dataset)?;
            let state_change = mk_obs_state_change(&["set_content"], &new_dataset_data);

            // prepare the reverse event (setting the original ID back)
            let reverse_at_path = [dataset_id.as_str(), "set_content"];
            let payload = orig_dataset_data.to_json_str();
            let reverse_event = mk_obs_event(&reverse_at_path, Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with("remove_var", at_path).is_some() {
            // get the payload - string encoding a new dataset data
            let var_id_str = Self::clone_payload_str(event, component_name)?;

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.remove_var_by_str(dataset_id.as_str(), var_id_str.as_str())?;
            let new_dataset = self.get_dataset(&dataset_id)?;
            let new_dataset_data = DatasetData::from_dataset(&dataset_id, new_dataset);
            let state_change = mk_obs_state_change(&["remove_var"], &new_dataset_data);

            // TODO: make this potentially reversible?
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else if Self::starts_with("set_var_id", at_path).is_some() {
            // get the payload - string for ChangeIdData
            let payload = Self::clone_payload_str(event, component_name)?;
            let id_change_data = ChangeIdData::from_json_str(&payload)?;
            let orig_id = id_change_data.original_id;
            let new_id = id_change_data.new_id;
            if orig_id == new_id {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move id from path to payload)
            let orig_dataset = self.get_dataset(&dataset_id)?;
            let orig_metadata = DatasetMetaData::from_dataset(&dataset_id, orig_dataset);
            self.set_var_id_by_str(dataset_id.as_str(), &orig_id, &new_id)?;
            let state_change = mk_obs_state_change(&["set_var_id"], &orig_metadata);

            // prepare the reverse event
            let reverse_at_path = [dataset_id.as_str(), "set_var_id"];
            let payload = ChangeIdData::new(&new_id, &orig_id).to_json_str();
            let reverse_event = mk_obs_event(&reverse_at_path, Some(&payload));
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with("push_obs", at_path).is_some() {
            // Adding particular observation to the end of a specific dataset
            // This is handled by the `Dataset` itself

            // the ID is valid (checked before), we can unwrap
            let dataset = self.datasets.get_mut(&dataset_id).unwrap();
            dataset.event_push_observation(event, dataset_id)
        } else if Self::starts_with("push_empty_obs", at_path).is_some() {
            // Adding new empty observation to the end of a specific dataset
            // This is handled by the `Dataset` itself

            // the ID is valid (checked before), we can unwrap
            let dataset = self.datasets.get_mut(&dataset_id).unwrap();
            dataset.event_push_empty_observation(event, dataset_id)
        } else if Self::starts_with("pop_obs", at_path).is_some() {
            // Removing last observation from the end of a specific dataset
            // This is handled by the `Dataset` itself

            // the ID is valid (checked before), we can unwrap
            let dataset = self.datasets.get_mut(&dataset_id).unwrap();
            dataset.event_pop_observation(event, dataset_id)
        } else {
            // Finally, remaining events must be some kind of modification of a specific observation
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
