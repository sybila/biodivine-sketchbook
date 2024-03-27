use crate::app::event::Event;
use crate::app::state::SessionState;
use crate::sketchbook::_tests_events::check_reverse;
use crate::sketchbook::data_structs::*;
use crate::sketchbook::ids::DatasetId;
use crate::sketchbook::observations::{DataCategory, Dataset, Observation, ObservationManager};
use crate::sketchbook::JsonSerde;

/// Prepare a simple dataset with 2 observations and 3 variables.
fn prepare_dataset1() -> Dataset {
    let obs1 = Observation::try_from_str("*11", "o1").unwrap();
    let obs2 = Observation::try_from_str("000", "o2").unwrap();
    let obs_list = vec![obs1, obs2];
    let var_names = vec!["a", "b", "c"];
    let obs_type = DataCategory::FixedPoint;
    Dataset::new(obs_list.clone(), var_names.clone(), obs_type).unwrap()
}

/// Prepare a simple dataset with 1 observation and 2 variables.
fn prepare_dataset2() -> Dataset {
    let obs1 = Observation::try_from_str("11", "o1").unwrap();
    let obs_list = vec![obs1];
    let var_names = vec!["v1", "v2"];
    let obs_type = DataCategory::Unspecified;
    Dataset::new(obs_list.clone(), var_names.clone(), obs_type).unwrap()
}

#[test]
/// Test adding and removing dataset via events.
fn test_add_remove_datasets() {
    let d1 = prepare_dataset1();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let manager_orig = manager.clone();
    assert_eq!(manager.num_datasets(), 1);

    // perform dataset add event
    let d2 = prepare_dataset2();
    let d2_id = DatasetId::new("d2").unwrap();
    let payload = DatasetData::from_dataset(&d2, &d2_id).to_json_str();
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
    let d1 = prepare_dataset1();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let manager_orig = manager.clone();

    // 1) event to set dataset's ID
    let full_path = ["observations", "d1", "set_id"];
    let event = Event::build(&full_path, Some("d2"));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    assert!(manager.get_dataset_by_str("d1").is_err());
    assert!(manager.get_dataset_by_str("d2").is_ok());
    check_reverse(&mut manager, &manager_orig, result, &["d2", "set_id"]);

    // 2) event to set dataset's type
    let new_type = DataCategory::Unspecified;
    let full_path = ["observations", "d1", "set_category"];
    let event = Event::build(&full_path, Some(&new_type.to_json_str()));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    let current_type = manager.get_dataset_by_str("d1").unwrap().category();
    assert_eq!(current_type, &new_type);
    check_reverse(&mut manager, &manager_orig, result, &["d1", "set_category"]);

    // 3) event to change dataset's inner "data"
    let d2 = prepare_dataset2();
    let d2_id = DatasetId::new("this_doesnt_matter").unwrap();
    let d2_data = DatasetData::from_dataset(&d2, &d2_id);
    let full_path = ["observations", "d1", "set_content"];
    let event = Event::build(&full_path, Some(&d2_data.to_json_str()));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    assert_eq!(manager.get_dataset_by_str("d1").unwrap().num_variables(), 2);
    check_reverse(&mut manager, &manager_orig, result, &["d1", "set_content"]);

    // 4) event to change one of dataset's variables
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
/// Test pushing/popping observations via events.
fn test_push_pop_observations() {
    let d1 = prepare_dataset1();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let d1_id = manager.get_dataset_id("d1").unwrap();
    let manager_orig = manager.clone();
    let orig_dataset = manager.get_dataset_by_str("d1").unwrap();
    assert_eq!(orig_dataset.num_observations(), 2);

    // perform observation push event
    let new_obs = Observation::try_from_str("111", "new_obs").unwrap();
    let payload = ObservationData::from_obs(&new_obs, &d1_id).to_json_str();
    let full_path = ["observations", "d1", "push_obs"];
    let event = Event::build(&full_path, Some(payload.as_str()));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    // check observation was added, test reverse action
    let modified_dataset = manager.get_dataset_by_str("d1").unwrap();
    let pushed_obs = manager.get_observation(&d1_id, new_obs.get_id()).unwrap();
    assert_eq!(modified_dataset.num_observations(), 3);
    assert_eq!(pushed_obs, &new_obs);
    check_reverse(&mut manager, &manager_orig, result, &["d1", "pop_obs"]);

    // perform observation pop event
    let full_path = ["observations", "d1", "pop_obs"];
    let event = Event::build(&full_path, None);
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    // check observation was removed, test reverse action
    let modified_dataset = manager.get_dataset_by_str("d1").unwrap();
    assert_eq!(modified_dataset.num_observations(), 1);
    check_reverse(&mut manager, &manager_orig, result, &["d1", "push_obs"]);
}

#[test]
/// Test removing an observation from a dataset via events.
fn test_remove_observations() {
    let d1 = prepare_dataset1();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let orig_dataset = manager.get_dataset_by_str("d1").unwrap();
    assert_eq!(orig_dataset.num_observations(), 2);
    assert_eq!(orig_dataset.get_observation_id(0).as_str(), "o1");

    // perform observation remove event (remove the first of two observations)
    let full_path = ["observations", "d1", "o1", "remove"];
    let event = Event::build(&full_path, None);
    manager.perform_event(&event, &full_path[1..]).unwrap();
    // check observation was removed
    let modified_dataset = manager.get_dataset_by_str("d1").unwrap();
    assert_eq!(modified_dataset.num_observations(), 1);
    assert_eq!(modified_dataset.get_observation_id(0).as_str(), "o2");
}

#[test]
/// Test setting various dataset fields via events.
fn test_set_observation_fields() {
    let d1 = prepare_dataset1();
    let mut manager = ObservationManager::from_datasets(vec![("d1", d1)]).unwrap();
    let d1_id = manager.get_dataset_id("d1").unwrap();
    let manager_orig = manager.clone();

    // 1) event to set observation's ID
    let full_path = ["observations", "d1", "o1", "set_id"];
    let event = Event::build(&full_path, Some("new_id"));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    assert!(manager.get_observation_by_str("d1", "o1").is_err());
    assert!(manager.get_observation_by_str("d1", "new_id").is_ok());
    let reverse_at_path = ["d1", "new_id", "set_id"];
    check_reverse(&mut manager, &manager_orig, result, &reverse_at_path);

    // 2) event to change observation's inner "data"
    let obs_original = manager.get_observation_by_str("d1", "o1").unwrap();
    assert_eq!(obs_original.num_zeros(), 0);
    let new_obs = Observation::try_from_str("000", "doesnt_matter").unwrap();
    let new_obs_data = ObservationData::from_obs(&new_obs, &d1_id);
    let full_path = ["observations", "d1", "o1", "set_content"];
    let event = Event::build(&full_path, Some(&new_obs_data.to_json_str()));
    let result = manager.perform_event(&event, &full_path[1..]).unwrap();
    let obs_modified = manager.get_observation_by_str("d1", "o1").unwrap();
    assert_eq!(obs_modified.num_zeros(), 3);
    check_reverse(
        &mut manager,
        &manager_orig,
        result,
        &["d1", "o1", "set_content"],
    );
}

#[test]
/// Test all of the refresh (getter) events.
fn test_refresh() {
    let d1 = prepare_dataset1();
    let d2 = prepare_dataset2();
    let dataset_list = vec![("d1", d1.clone()), ("d2", d2)];
    let manager = ObservationManager::from_datasets(dataset_list).unwrap();

    // test getter for all datasets
    let full_path = ["observations".to_string(), "get_all_datasets".to_string()];
    let event = manager.refresh(&full_path, &["get_all_datasets"]).unwrap();
    let dataset_list: Vec<DatasetData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(dataset_list.len(), 2);
    assert_eq!(&dataset_list[0].id, "d1");

    // test getter for a single dataset
    let full_path = [
        "observations".to_string(),
        "get_dataset".to_string(),
        "d1".to_string(),
    ];
    let event = manager.refresh(&full_path, &["get_dataset", "d1"]).unwrap();
    let dataset_data = DatasetData::from_json_str(&event.payload.unwrap()).unwrap();
    assert_eq!(dataset_data.to_dataset().unwrap(), d1);

    // test getter for a single observation
    let full_path = [
        "observations".to_string(),
        "get_observation".to_string(),
        "d1".to_string(),
        "o2".to_string(),
    ];
    let at_path = ["get_observation", "d1", "o2"];
    let event = manager.refresh(&full_path, &at_path).unwrap();
    let obs_data = ObservationData::from_json_str(&event.payload.unwrap()).unwrap();
    let expected_obs = d1.get_observation_on_idx(1).unwrap();
    assert_eq!(&obs_data.to_observation().unwrap(), expected_obs);
}
