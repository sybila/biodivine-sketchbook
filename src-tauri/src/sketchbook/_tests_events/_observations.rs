use crate::app::event::Event;
use crate::app::state::SessionState;
use crate::sketchbook::JsonSerde;
use crate::sketchbook::_tests_events::{check_reverse, stringify_path};
use crate::sketchbook::data_structs::*;
use crate::sketchbook::ids::DatasetId;
use crate::sketchbook::observations::{Dataset, Observation, ObservationManager};

/// Prepare a simple dataset with 3 variables and 2 observations.
fn prepare_dataset_3v_2o() -> Dataset {
    let obs1 = Observation::try_from_str("*11", "o1").unwrap();
    let obs2 = Observation::try_from_str("000", "o2").unwrap();
    let obs_list = vec![obs1, obs2];
    let var_names = vec!["a", "b", "c"];
    Dataset::new("dataset_3v_2o", obs_list.clone(), var_names.clone()).unwrap()
}

/// Prepare a simple dataset with 2 variables and 1 observation.
fn prepare_dataset_2v_1o() -> Dataset {
    let obs1 = Observation::try_from_str("11", "o1").unwrap();
    let obs_list = vec![obs1];
    let var_names = vec!["v1", "v2"];
    Dataset::new("dataset_2v_1o", obs_list.clone(), var_names.clone()).unwrap()
}

#[test]
/// Test adding and removing dataset via events.
fn test_add_remove_datasets() {
    let d1 = prepare_dataset_3v_2o();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let manager_orig = manager.clone();
    assert_eq!(manager.num_datasets(), 1);

    // perform dataset add event
    let d2 = prepare_dataset_2v_1o();
    let d2_id = DatasetId::new("d2").unwrap();
    let payload = DatasetData::from_dataset(&d2_id, &d2).to_json_str();
    let full_path = ["observations", "add"];
    let event = Event::build(&full_path, Some(payload.as_str()));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    // check dataset was added, test reverse action
    assert_eq!(manager.num_datasets(), 2);
    assert_eq!(manager.get_dataset_by_str("d2").unwrap(), &d2);
    check_reverse(&mut manager, &manager_orig, result, &["d2", "remove"]);

    // perform dataset remove event
    let full_path = ["observations", "d1", "remove"];
    let event = Event::build(&full_path, None);
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    // check dataset was removed, test reverse action
    assert_eq!(manager.num_datasets(), 0);
    assert!(manager.get_dataset_by_str("d1").is_err());
    check_reverse(&mut manager, &manager_orig, result, &["add"]);
}

#[test]
/// Test setting various dataset fields via events.
fn test_set_dataset_fields() {
    let d1 = prepare_dataset_3v_2o();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let manager_orig = manager.clone();

    // 1) event to set dataset's ID
    let full_path = ["observations", "d1", "set_id"];
    let event = Event::build(&full_path, Some("d2"));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    assert!(manager.get_dataset_by_str("d1").is_err());
    assert!(manager.get_dataset_by_str("d2").is_ok());
    check_reverse(&mut manager, &manager_orig, result, &["d2", "set_id"]);

    // 2) event to change dataset's inner "data"
    let d2 = prepare_dataset_2v_1o();
    let d2_id = DatasetId::new("this_doesnt_matter").unwrap();
    let d2_data = DatasetData::from_dataset(&d2_id, &d2);
    let full_path = ["observations", "d1", "set_content"];
    let event = Event::build(&full_path, Some(&d2_data.to_json_str()));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    assert_eq!(manager.get_dataset_by_str("d1").unwrap().num_variables(), 2);
    check_reverse(&mut manager, &manager_orig, result, &["d1", "set_content"]);

    // 3) event to change one of dataset's variables
    let full_path = ["observations", "d1", "set_var_id"];
    let payload = ChangeIdData::new("a", "xyz").to_json_str();
    let event = Event::build(&full_path, Some(&payload));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    let d1_ref = manager.get_dataset_by_str("d1").unwrap();
    assert!(d1_ref.get_var_id("a").is_err());
    assert!(d1_ref.get_var_id("xyz").is_ok());
    check_reverse(&mut manager, &manager_orig, result, &["d1", "set_var_id"]);
}

#[test]
/// Test pushing observations via event.
fn test_push_new_observation() {
    let d1 = prepare_dataset_3v_2o();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let d1_id = manager.get_dataset_id("d1").unwrap();
    let orig_dataset = manager.get_dataset_by_str("d1").unwrap();
    assert_eq!(orig_dataset.num_observations(), 2);

    // push empty observation
    let full_path = ["observations", "d1", "push_empty_obs"];
    let event = Event::build(&full_path, None);
    manager.perform_event(&event, &full_path[1..]).unwrap();
    // check observation was added, test reverse action
    let modified_dataset = manager.get_dataset_by_str("d1").unwrap();
    let obs_id = modified_dataset.get_obs_id(2);
    let expected_obs =
        Observation::new_full_unspecified(modified_dataset.num_variables(), obs_id.as_str())
            .unwrap();
    let pushed_obs = manager.get_obs(&d1_id, obs_id).unwrap();
    assert_eq!(modified_dataset.num_observations(), 3);
    assert_eq!(pushed_obs, &expected_obs);
}

#[test]
/// Test removing an observation from a dataset via events.
fn test_remove_observations() {
    let d1 = prepare_dataset_3v_2o();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let orig_dataset = manager.get_dataset_by_str("d1").unwrap();
    assert_eq!(orig_dataset.num_observations(), 2);
    assert_eq!(orig_dataset.get_obs_id(0).as_str(), "o1");

    // perform observation remove event (remove the first of two observations)
    let full_path = ["observations", "d1", "o1", "remove"];
    let event = Event::build(&full_path, None);
    manager.perform_event(&event, &full_path[1..]).unwrap();
    // check observation was removed
    let modified_dataset = manager.get_dataset_by_str("d1").unwrap();
    assert_eq!(modified_dataset.num_observations(), 1);
    assert_eq!(modified_dataset.get_obs_id(0).as_str(), "o2");
}

#[test]
/// Test setting various dataset fields via events.
fn test_set_observation_fields() {
    let d1 = prepare_dataset_3v_2o();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let d1_id = manager.get_dataset_id("d1").unwrap();
    let manager_orig = manager.clone();

    // 1) event to set observation's ID
    let full_path = ["observations", "d1", "o1", "set_id"];
    let event = Event::build(&full_path, Some("new_id"));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    assert!(manager.get_obs_by_str("d1", "o1").is_err());
    assert!(manager.get_obs_by_str("d1", "new_id").is_ok());
    let reverse_at_path = ["d1", "new_id", "set_id"];
    check_reverse(&mut manager, &manager_orig, result, &reverse_at_path);

    // 2) event to change observation's inner "data"
    let obs_original = manager.get_obs_by_str("d1", "o1").unwrap();
    assert_eq!(obs_original.num_zeros(), 0);
    let new_obs = Observation::try_from_str("000", "doesnt_matter").unwrap();
    let new_obs_data = ObservationData::from_obs(&new_obs, &d1_id);
    let full_path = ["observations", "d1", "o1", "set_data"];
    let event = Event::build(&full_path, Some(&new_obs_data.to_json_str()));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    let obs_modified = manager.get_obs_by_str("d1", "o1").unwrap();
    assert_eq!(obs_modified.num_zeros(), 3);
    check_reverse(
        &mut manager,
        &manager_orig,
        result,
        &["d1", "o1", "set_data"],
    );
}

#[test]
/// Test all of the refresh (getter) events.
fn test_refresh() {
    let d1 = prepare_dataset_3v_2o();
    let d2 = prepare_dataset_2v_1o();
    let dataset_list = vec![("d1", d1.clone()), ("d2", d2)];
    let manager = ObservationManager::from_datasets(dataset_list).unwrap();

    // test getter for all datasets
    let full_path = stringify_path(&["sketch", "observations", "get_all_datasets"]);
    let event = manager.refresh(&full_path, &["get_all_datasets"]).unwrap();
    let dataset_list: Vec<DatasetData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(dataset_list.len(), 2);
    assert_eq!(&dataset_list[0].id, "d1");

    // test getter for a single dataset
    let full_path = stringify_path(&["sketch", "observations", "get_dataset", "d1"]);
    let event = manager.refresh(&full_path, &["get_dataset", "d1"]).unwrap();
    let dataset_data = DatasetData::from_json_str(&event.payload.unwrap()).unwrap();
    assert_eq!(dataset_data.to_dataset().unwrap(), d1);

    // test getter for a single observation
    let full_path = stringify_path(&["sketch", "observations", "get_observation", "d1", "o2"]);
    let at_path = ["get_observation", "d1", "o2"];
    let event = manager.refresh(&full_path, &at_path).unwrap();
    let obs_data = ObservationData::from_json_str(&event.payload.unwrap()).unwrap();
    let expected_obs = d1.get_obs_on_idx(1).unwrap();
    assert_eq!(&obs_data.to_observation().unwrap(), expected_obs);
}
