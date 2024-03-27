use crate::app::event::Event;
use crate::app::state::{Consumed, SessionState};
use crate::sketchbook::_tests_events::check_reverse;
use crate::sketchbook::data_structs::*;
use crate::sketchbook::layout::NodePosition;
use crate::sketchbook::{Essentiality, JsonSerde, ModelState, Monotonicity, VarId};

#[test]
fn test_add_var() {
    let mut model = ModelState::new();
    let var_a = model.generate_var_id("a");
    model.add_var(var_a, "a").unwrap();
    let model_orig = model.clone();
    assert_eq!(model.num_vars(), 1);

    // test variable add event
    let payload = VariableData::new("b", "b", "").to_json_str();
    let full_path = ["model", "variable", "add"];
    let event = Event::build(&full_path, Some(payload.as_str()));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check var was added correctly, and test reverse event
    assert_eq!(model.num_vars(), 2);
    assert_eq!(model.get_var_id("b").unwrap(), VarId::new("b").unwrap());
    let reverse_at_path = ["variable", "b", "remove"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
/// Test removing a variable that has no regulations and its node is in default position.
fn test_remove_var_simple() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let model_orig = model.clone();

    // test variable remove event
    let full_path = ["model", "variable", "a", "remove"];
    let event = Event::build(&full_path, None);
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check var was removed - result should be a simple `Consumed::Reversible` object
    assert_eq!(model.num_vars(), 1);
    check_reverse(&mut model, &model_orig, result, &["variable", "add"]);
}

#[test]
/// Test removing a variable that has regulations and its node is in non-default position.
fn test_remove_var_complex() {
    // make a model where var `a` is part of regulations, and its position is changed
    let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
    let var_a = model.get_var_id("a").unwrap();
    let regulations = vec!["a -> a", "a -> b", "b -> b"];
    model.add_multiple_regulations(regulations).unwrap();
    let layout_id = &ModelState::get_default_layout_id();
    model.update_position(layout_id, &var_a, 1., 1.).unwrap();

    // expected result
    let mut model_expected = ModelState::new();
    model_expected.add_var_by_str("b", "b").unwrap();
    model_expected.add_regulation_by_str("b -> b").unwrap();

    // test variable remove event
    let full_path = ["model", "variable", "a", "remove"];
    let event = Event::build(&full_path, None);
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // result should be a `Consumed::Restart` object with a vector of events which should simulate the individual
    // steps of the variable's removal
    if let Consumed::Restart(mut sub_events) = result {
        // there should be 2 events for regulations removal, 1 for re-position, and final one for var removal
        assert_eq!(sub_events.len(), 4);
        sub_events.reverse();
        for e in sub_events {
            let mut full_path = e.path.clone();
            full_path.remove(0);
            let full_path_str: Vec<&str> = full_path.iter().map(|s| s.as_str()).collect();
            println!("{:?}", e);
            model.perform_event(&e, &full_path_str).unwrap();
        }
        assert_eq!(model, model_expected);
    } else {
        unreachable!();
    }
}

#[test]
fn test_set_var_name() {
    let orig_name = "a_name";
    let mut model = ModelState::new_from_vars(vec![("a", orig_name)]).unwrap();
    let var_a = model.get_var_id("a").unwrap();
    let new_name = "new_name";
    let model_orig = model.clone();
    assert_eq!(model.get_var_name(&var_a).unwrap(), orig_name);

    // test variable rename event
    let full_path = ["model", "variable", var_a.as_str(), "set_name"];
    let event = Event::build(&full_path, Some(new_name));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check var was renamed correctly, and test the reverse event
    assert_eq!(model.get_var_name(&var_a).unwrap(), new_name);
    check_reverse(&mut model, &model_orig, result, &full_path[1..]);
}

#[test]
fn test_set_var_id() {
    let mut model = ModelState::new_from_vars(vec![("a", "a_name")]).unwrap();
    let var_a = model.get_var_id("a").unwrap();
    let model_orig = model.clone();

    // test id change event
    let new_id = model.generate_var_id("b");
    let full_path = ["model", "variable", var_a.as_str(), "set_id"];
    let event = Event::build(&full_path, Some(new_id.as_str()));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check id changed correctly, and test the reverse event
    assert!(!model.is_valid_var_id(&var_a));
    assert!(model.is_valid_var_id(&new_id));
    let reverse_at_path = ["variable", new_id.as_str(), "set_id"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
fn test_set_update_fn() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let model_orig = model.clone();
    let expression = "a => b";
    let var_a = model.get_var_id("a").unwrap();

    // test update fn modification event
    let full_path = ["model", "variable", var_a.as_str(), "set_update_fn"];
    let event = Event::build(&full_path, Some(expression));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check that update fn was set correctly, and test the reverse event
    assert_eq!(model.get_update_fn_string(&var_a).unwrap(), expression);
    let reverse_at_path = ["variable", var_a.as_str(), "set_update_fn"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
fn test_invalid_var_events() {
    let mut model = ModelState::new();
    let var_id = model.generate_var_id("a");
    model.add_var(var_id.clone(), "a-name").unwrap();
    let model_orig = model.clone();

    // adding variable `a` again
    let full_path = ["model", "variable", "add"];
    let event = Event::build(&full_path, Some("a"));
    assert!(model.perform_event(&event, &full_path[1..]).is_err());
    assert_eq!(model, model_orig);

    // removing variable with wrong id
    let full_path = ["model", "variable", "b", "remove"];
    let event = Event::build(&full_path, None);
    assert!(model.perform_event(&event, &full_path[1..]).is_err());
    assert_eq!(model, model_orig);

    // variable rename event with wrong id
    let full_path = ["model", "variable", "b", "set_name"];
    let event = Event::build(&full_path, Some("new_name"));
    assert!(model.perform_event(&event, &full_path[1..]).is_err());
    assert_eq!(model, model_orig);
}

#[test]
fn test_add_reg() {
    let variables = vec![("a", "a"), ("b", "b")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let model_orig = model.clone();

    // test regulation add event
    let full_path = ["model", "regulation", "add"];
    let regulation_data = RegulationData::try_from_reg_str("a -> b").unwrap();
    let event = Event::build(&full_path, Some(&regulation_data.to_json_str()));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check that regulation was added correctly, and test the reverse event
    assert_eq!(model.num_regulations(), 1);
    let reverse_at_path = ["regulation", "a", "b", "remove"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
fn test_change_reg_sign() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let regulations = vec!["a -> a", "a -> b", "b -> a"];
    model.add_multiple_regulations(regulations).unwrap();
    let model_orig = model.clone();

    // test event for changing regulation's sign
    let full_path = ["model", "regulation", "a", "b", "set_sign"];
    let new_sign = Monotonicity::Inhibition.to_json_str();
    let event = Event::build(&full_path, Some(&new_sign));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check that regulation's sign was set correctly, and test the reverse event
    let reg = model.get_regulation_by_str("a", "b").unwrap();
    assert_eq!(reg.get_sign(), &Monotonicity::Inhibition);
    let reverse_at_path = ["regulation", "a", "b", "set_sign"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
fn test_change_reg_essentiality() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let regulations = vec!["a -> a", "a -> b", "b -> a"];
    model.add_multiple_regulations(regulations).unwrap();
    let model_orig = model.clone();

    // test event for changing regulation's essentiality
    let full_path = ["model", "regulation", "a", "b", "set_essentiality"];
    let new_essentiality = Essentiality::False.to_json_str();
    let event = Event::build(&full_path, Some(&new_essentiality));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check that regulation's essentiality was set correctly, and test the reverse event
    let reg = model.get_regulation_by_str("a", "b").unwrap();
    assert_eq!(reg.get_essentiality(), &Essentiality::False);
    let reverse_at_path = ["regulation", "a", "b", "set_essentiality"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
fn test_remove_reg() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    model.add_regulation_by_str("a -> b").unwrap();
    let model_orig = model.clone();

    // test regulation remove event
    let full_path = ["model", "regulation", "a", "b", "remove"];
    let event = Event::build(&full_path, None);
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check that regulation was removed correctly, and test the reverse event
    assert_eq!(model.num_regulations(), 0);
    check_reverse(&mut model, &model_orig, result, &["regulation", "add"]);
}

#[test]
fn test_change_position() {
    let mut model = ModelState::new();
    let layout_id = ModelState::get_default_layout_id();
    let var_id = model.generate_var_id("a");
    model.add_var(var_id.clone(), "a_name").unwrap();
    let model_orig = model.clone();

    // test position change event
    let payload = LayoutNodeData::new(layout_id.as_str(), var_id.as_str(), 2.5, 0.4).to_json_str();
    let full_path = ["model", "layout", layout_id.as_str(), "update_position"];
    let event = Event::build(&full_path, Some(payload.as_str()));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check position changed correctly, and test reverse event
    let new_position = model.get_node_position(&layout_id, &var_id).unwrap();
    assert_eq!(new_position, &NodePosition(2.5, 0.4));
    check_reverse(&mut model, &model_orig, result, &full_path[1..]);
}

#[test]
fn test_change_fn_arg_monotonicity() {
    let mut model = ModelState::new();
    let f = model.generate_uninterpreted_fn_id("f");
    model.add_new_uninterpreted_fn(f.clone(), "f", 2).unwrap();
    let model_orig = model.clone();

    // test event for changing uninterpreted fn's monotonicity
    let full_path = ["model", "uninterpreted_fn", f.as_str(), "set_monotonicity"];
    let change_data = ChangeArgMonotoneData::new(1, Monotonicity::Dual).to_json_str();
    let event = Event::build(&full_path, Some(&change_data));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check if the argument's monotonicity changed correctly, and test reverse event
    let uninterpreted_fn = model.get_uninterpreted_fn(&f).unwrap();
    assert_eq!(uninterpreted_fn.get_monotonic(1), &Monotonicity::Dual);
    let reverse_at_path = ["uninterpreted_fn", f.as_str(), "set_monotonicity"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
fn test_change_fn_arg_essentiality() {
    let mut model = ModelState::new();
    let f = model.generate_uninterpreted_fn_id("f");
    model.add_new_uninterpreted_fn(f.clone(), "f", 2).unwrap();
    let model_orig = model.clone();

    // test event for changing uninterpreted fn's expression
    let full_path = ["model", "uninterpreted_fn", f.as_str(), "set_essentiality"];
    let change_data = ChangeArgEssentialData::new(1, Essentiality::True).to_json_str();
    let event = Event::build(&full_path, Some(&change_data));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check if the argument's essentiality changed correctly, and test reverse event
    let uninterpreted_fn = model.get_uninterpreted_fn(&f).unwrap();
    assert_eq!(uninterpreted_fn.get_essential(1), &Essentiality::True);
    let reverse_at_path = ["uninterpreted_fn", f.as_str(), "set_essentiality"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
fn test_change_fn_expression() {
    let mut model = ModelState::new();
    let f = model.generate_uninterpreted_fn_id("f");
    model.add_new_uninterpreted_fn(f.clone(), "f", 2).unwrap();
    let model_orig = model.clone();

    // test event for changing uninterpreted fn's expression
    let expression = "var0 | var1";
    let full_path = ["model", "uninterpreted_fn", f.as_str(), "set_expression"];
    let event = Event::build(&full_path, Some(expression));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check if the expression changed correctly, and test reverse event
    let uninterpreted_fn = model.get_uninterpreted_fn(&f).unwrap();
    assert_eq!(uninterpreted_fn.get_fn_expression(), expression);
    let reverse_at_path = ["uninterpreted_fn", f.as_str(), "set_expression"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
fn test_refresh() {
    let mut model = ModelState::new();
    let layout_id = ModelState::get_default_layout_id().to_string();
    let var_a = model.generate_var_id("a");
    let f = model.generate_uninterpreted_fn_id("f");
    let expression = "a | a";

    model.add_var(var_a.clone(), "a_name").unwrap();
    model.add_regulation_by_str("a -> a").unwrap();
    model
        .add_new_uninterpreted_fn(f.clone(), "f_name", 2)
        .unwrap();
    model.set_update_fn(&var_a, expression).unwrap();

    // test variables getter
    let full_path = ["model".to_string(), "get_variables".to_string()];
    let event = model.refresh(&full_path, &["get_variables"]).unwrap();
    let var_list: Vec<VariableData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(var_list.len(), 1);
    assert_eq!(var_list[0].name, "a_name".to_string());
    assert_eq!(var_list[0].id, var_a.to_string());
    assert_eq!(var_list[0].update_fn, expression);

    // test regulations getter
    let full_path = ["model".to_string(), "get_regulations".to_string()];
    let event = model.refresh(&full_path, &["get_regulations"]).unwrap();
    let reg_list: Vec<RegulationData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(reg_list.len(), 1);
    assert_eq!(reg_list[0].target, var_a.to_string());
    assert_eq!(reg_list[0].regulator, var_a.to_string());

    // test layouts getter
    let full_path = ["model".to_string(), "get_layouts".to_string()];
    let event = model.refresh(&full_path, &["get_layouts"]).unwrap();
    let layout_list: Vec<LayoutData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(layout_list.len(), 1);
    assert_eq!(layout_list[0].id, layout_id.clone());

    // test layout nodes getter
    let full_path = [
        "model".to_string(),
        "get_layout_nodes".to_string(),
        layout_id.clone(),
    ];
    let event = model
        .refresh(&full_path, &["get_layout_nodes", &layout_id])
        .unwrap();
    let node_list: Vec<LayoutNodeData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(node_list.len(), 1);
    assert_eq!(node_list[0].variable, var_a.to_string());

    // test uninterpreted functions getter
    let full_path = ["model".to_string(), "get_uninterpreted_fns".to_string()];
    let event = model
        .refresh(&full_path, &["get_uninterpreted_fns"])
        .unwrap();
    let uninterpreted_fn_list: Vec<UninterpretedFnData> =
        serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(uninterpreted_fn_list.len(), 1);
    assert_eq!(uninterpreted_fn_list[0].id, f.to_string());
}
