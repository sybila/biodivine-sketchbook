use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::sketchbook::data_structs::DatasetData;
use crate::sketchbook::event_utils::{make_refresh_event, make_reversible, make_state_change};
use crate::sketchbook::observations::ObservationManager;
use crate::sketchbook::DatasetId;
use std::str::FromStr;

impl SessionHelper for ObservationManager {}

impl SessionState for ObservationManager {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        let component_name = "observation_manager";

        // there is either adding of a new dataset, or editing/removing of an existing one
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
        match at_path.first() {
            Some(&"get_all_datasets") => {
                let dataset_list: Vec<DatasetData> = self
                    .datasets
                    .iter()
                    .map(|(id, dataset)| DatasetData::from_dataset(dataset, id))
                    .collect();
                make_refresh_event(full_path, dataset_list)
            }
            Some(&"get_dataset") => {
                // path specifies dataset's ID
                Self::assert_path_length(at_path, 1, "observation_manager/datasets")?;
                let dataset_id_str = at_path.first().unwrap();
                let dataset_id = self.get_dataset_id(dataset_id_str)?;
                let dataset = self.get_dataset(&dataset_id)?;
                let dataset_data = DatasetData::from_dataset(dataset, &dataset_id);
                let payload = Some(dataset_data.to_string());

                // remove the id from the path
                let mut path = full_path.to_vec();
                path.pop();

                Ok(Event { path, payload })
            }
            Some(&"get_observation") => {
                // path specifies dataset's ID and observation's ID
                Self::assert_path_length(at_path, 2, "observation_manager/datasets/observations")?;
                // todo: return ObservationData
                todo!()
            }
            _ => Self::invalid_path_error_generic(at_path),
        }
    }
}

/// Implementation for events related to `datasets`.
impl ObservationManager {
    /// Perform event of adding a new `dataset` component to this `ObservationManager`.
    pub(super) fn event_add_dataset(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "observation_manager/datasets";

        // get payload components and perform the event
        let payload = Self::clone_payload_str(event, component_name)?;
        let _dataset_data = DatasetData::from_str(payload.as_str())?;
        // todo: parse and add the dataset
        todo!()
    }

    /// Perform event of modifying or removing existing `dataset` component of this
    /// `ObservationManager`.
    pub(super) fn event_modify_dataset(
        &mut self,
        event: &Event,
        at_path: &[&str],
        dataset_id: DatasetId,
    ) -> Result<Consumed, DynError> {
        let component_name = "observation_manager/datasets";

        // there is either editing whole dataset directly with `at_path` being [<ACTION>]
        // or editing specific observation with `at_path` being ["observation_id", ...]

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
            // todo: set dataset's ID
            todo!()
        } else if Self::starts_with("change_data", at_path).is_some() {
            // todo: change dataset's whole observation list to a new list
            todo!()
        } else {
            // otherwise we edit specific observation with `at_path` being ["observation_id", ...]
            // todo: sent the event down to dataset
            todo!()
        }
    }
}
