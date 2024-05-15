use crate::app::state::{Consumed, SessionState};
use crate::sketchbook::_tests_events::{check_reverse, stringify_path};
use crate::sketchbook::data_structs::*;
use crate::sketchbook::event_utils::mk_model_event;
use crate::sketchbook::ids::VarId;
use crate::sketchbook::layout::NodePosition;
use crate::sketchbook::model::{Essentiality, ModelState, Monotonicity};
use crate::sketchbook::JsonSerde;

#[test]
/// Test adding variable via events.
fn test_add_var() {
    let variables = vec![("a", "a")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let model_orig = model.clone();
    assert_eq!(model.num_vars(), 1);

    // test variable add event
    let payload = VariableData::new("b", "b", "").to_json_str();
    let at_path = ["variable", "add"];
    let event = mk_model_event(&at_path, Some(&payload));
    let result = model.perform_event(&event, &at_path).unwrap();

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
    let at_path = ["variable", "a", "remove"];
    let event = mk_model_event(&at_path, None);
    let result = model.perform_event(&event, &at_path).unwrap();

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
    let mut model_expected = ModelState::new_empty();
    model_expected.add_var_by_str("b", "b").unwrap();
    model_expected.add_regulation_by_str("b -> b").unwrap();

    // test variable remove event
    let at_path = ["variable", "a", "remove"];
    let event = mk_model_event(&at_path, None);
    let result = model.perform_event(&event, &at_path).unwrap();

    // result should be a `Consumed::Restart` object with a vector of events which should simulate the individual
    // steps of the variable's removal
    if let Consumed::Restart(mut sub_events) = result {
        // there should be 2 events for regulations removal, 1 for re-position, and final one for var removal
        assert_eq!(sub_events.len(), 4);
        sub_events.reverse();
        for e in sub_events {
            let at_path_str: Vec<&str> = e.path[2..].iter().map(|s| s.as_str()).collect();
            println!("{:?}", e);
            let res_inner = model.perform_event(&e, &at_path_str).unwrap();
            if let Consumed::Restart(sub_events) = res_inner {
                // reg removal can be composed of up to three sub-events (static props)
                assert_eq!(sub_events.len(), 3);
                // properties do not interest us here, just take the last event for actual regulation
                let reg_event = sub_events.first().unwrap();
                let reg_at_path: Vec<&str> =
                    reg_event.path[2..].iter().map(|s| s.as_str()).collect();
                model.perform_event(&reg_event, &reg_at_path).unwrap();
            }
        }
        assert_eq!(model, model_expected);
    } else {
        unreachable!();
    }
}

#[test]
/// Test setting variable's name and ID via events.
fn test_set_var_name_id() {
    let orig_name = "a_name";
    let mut model = ModelState::new_from_vars(vec![("a", orig_name)]).unwrap();
    let var_a = model.get_var_id("a").unwrap();
    let new_name = "new_name";
    let model_orig = model.clone();
    assert_eq!(model.get_var_name(&var_a).unwrap(), orig_name);

    // test variable rename event
    let at_path = ["variable", var_a.as_str(), "set_name"];
    let event = mk_model_event(&at_path, Some(new_name));
    let result = model.perform_event(&event, &at_path).unwrap();
    // check var was renamed correctly, and test the reverse event
    assert_eq!(model.get_var_name(&var_a).unwrap(), new_name);
    check_reverse(&mut model, &model_orig, result, &at_path);

    // test id change event
    let new_id = model.generate_var_id("b");
    let at_path = ["variable", var_a.as_str(), "set_id"];
    let event = mk_model_event(&at_path, Some(new_id.as_str()));
    let result = model.perform_event(&event, &at_path).unwrap();
    // check id changed correctly, and test the reverse event
    assert!(!model.is_valid_var_id(&var_a));
    assert!(model.is_valid_var_id(&new_id));
    let reverse_at_path = ["variable", new_id.as_str(), "set_id"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
/// Test setting variable's update function via event.
fn test_set_update_fn() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let model_orig = model.clone();
    let expression = "a => b";
    let var_a = model.get_var_id("a").unwrap();

    // test update fn modification event
    let at_path = ["variable", var_a.as_str(), "set_update_fn"];
    let event = mk_model_event(&at_path, Some(expression));
    let result = model.perform_event(&event, &at_path).unwrap();

    // check that update fn was set correctly, and test the reverse event
    assert_eq!(model.get_update_fn_string(&var_a).unwrap(), expression);
    let reverse_at_path = ["variable", var_a.as_str(), "set_update_fn"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
/// Test that several kinds of invalid operations fail successfully.
fn test_invalid_var_events() {
    let mut model = ModelState::new_empty();
    let var_id = model.generate_var_id("a");
    model.add_var(var_id.clone(), "a-name").unwrap();
    let model_orig = model.clone();

    // adding variable `a` again
    let at_path = ["variable", "add"];
    let event = mk_model_event(&at_path, Some("a"));
    assert!(model.perform_event(&event, &at_path).is_err());
    assert_eq!(model, model_orig);

    // removing variable with wrong id
    let at_path = ["variable", "b", "remove"];
    let event = mk_model_event(&at_path, None);
    assert!(model.perform_event(&event, &at_path).is_err());
    assert_eq!(model, model_orig);

    // variable rename event with wrong id
    let at_path = ["variable", "b", "set_name"];
    let event = mk_model_event(&at_path, Some("new_name"));
    assert!(model.perform_event(&event, &at_path).is_err());
    assert_eq!(model, model_orig);
}

#[test]
/// Test adding regulation via (raw) event.
/// todo: add complex version that adds regulation which requires adding static properties
fn test_add_reg_simple() {
    let variables = vec![("a", "a"), ("b", "b")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let model_orig = model.clone();

    // test regulation add event
    let at_path = ["regulation", "add_raw"];
    let regulation_data = RegulationData::try_from_reg_str("a -?? b").unwrap();
    let event = mk_model_event(&at_path, Some(&regulation_data.to_json_str()));
    let result = model.perform_event(&event, &at_path).unwrap();

    // check that regulation was added correctly, and test the reverse event
    assert_eq!(model.num_regulations(), 1);
    let reverse_at_path = ["regulation", "a", "b", "remove_raw"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
/// Test changing regulation's monotonicity and essentiality via event.
/// todo: add complex version which requires changes in static properties
fn test_change_reg_sign_essentiality() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let regulations = vec!["a -> a", "a -> b", "b -> a"];
    model.add_multiple_regulations(regulations).unwrap();
    let model_orig = model.clone();

    // test event for changing regulation's sign
    let at_path = ["regulation", "a", "b", "set_sign_raw"];
    let new_sign = Monotonicity::Inhibition.to_json_str();
    let event = mk_model_event(&at_path, Some(&new_sign));
    let result = model.perform_event(&event, &at_path).unwrap();
    // check that regulation's sign was set correctly, and test the reverse event
    let reg = model.get_regulation_by_str("a", "b").unwrap();
    assert_eq!(reg.get_sign(), &Monotonicity::Inhibition);
    let reverse_at_path = ["regulation", "a", "b", "set_sign_raw"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);

    // test event for changing regulation's essentiality
    let at_path = ["regulation", "a", "b", "set_essentiality_raw"];
    let new_essentiality = Essentiality::False.to_json_str();
    let event = mk_model_event(&at_path, Some(&new_essentiality));
    let result = model.perform_event(&event, &at_path).unwrap();
    // check that regulation's essentiality was set correctly, and test the reverse event
    let reg = model.get_regulation_by_str("a", "b").unwrap();
    assert_eq!(reg.get_essentiality(), &Essentiality::False);
    let reverse_at_path = ["regulation", "a", "b", "set_essentiality_raw"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
/// Test removing regulation via (raw) event.
/// todo: add complex version that removes regulation which requires removing static properties
fn test_remove_reg_simple() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    model.add_regulation_by_str("a -?? b").unwrap();
    let model_orig = model.clone();

    // test regulation remove event
    let at_path = ["regulation", "a", "b", "remove_raw"];
    let event = mk_model_event(&at_path, None);
    let result = model.perform_event(&event, &at_path).unwrap();

    // check that regulation was removed correctly, and test the reverse event
    assert_eq!(model.num_regulations(), 0);
    check_reverse(&mut model, &model_orig, result, &["regulation", "add_raw"]);
}

#[test]
/// Test changing position of a layout node via event.
fn test_change_position() {
    let mut model = ModelState::new_empty();
    let layout_id = ModelState::get_default_layout_id();
    let var_id = model.generate_var_id("a");
    model.add_var(var_id.clone(), "a_name").unwrap();
    let model_orig = model.clone();

    // test position change event
    let payload = LayoutNodeData::new(layout_id.as_str(), var_id.as_str(), 2.5, 0.4).to_json_str();
    let at_path = ["layout", layout_id.as_str(), "update_position"];
    let event = mk_model_event(&at_path, Some(&payload));
    let result = model.perform_event(&event, &at_path).unwrap();

    // check position changed correctly, and test reverse event
    let new_position = model.get_node_position(&layout_id, &var_id).unwrap();
    assert_eq!(new_position, &NodePosition(2.5, 0.4));
    check_reverse(&mut model, &model_orig, result, &at_path);
}

#[test]
/// Test changing monotonicity and essentiality of uninterpreted function's argument via event.
fn test_change_fn_arg_monotonicity_essentiality() {
    let mut model = ModelState::new_empty();
    let f = model.generate_uninterpreted_fn_id("f");
    model.add_empty_uninterpreted_fn(f.clone(), "f", 2).unwrap();
    let model_orig = model.clone();

    // test event for changing uninterpreted fn's monotonicity
    let at_path = ["uninterpreted_fn", f.as_str(), "set_monotonicity"];
    let change_data = ChangeArgMonotoneData::new(1, Monotonicity::Dual).to_json_str();
    let event = mk_model_event(&at_path, Some(&change_data));
    let result = model.perform_event(&event, &at_path).unwrap();
    // check if the argument's monotonicity changed correctly, and test reverse event
    let uninterpreted_fn = model.get_uninterpreted_fn(&f).unwrap();
    assert_eq!(uninterpreted_fn.get_monotonic(1), &Monotonicity::Dual);
    let reverse_at_path = ["uninterpreted_fn", f.as_str(), "set_monotonicity"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);

    // test event for changing uninterpreted fn's expression
    let at_path = ["uninterpreted_fn", f.as_str(), "set_essentiality"];
    let change_data = ChangeArgEssentialData::new(1, Essentiality::True).to_json_str();
    let event = mk_model_event(&at_path, Some(&change_data));
    let result = model.perform_event(&event, &at_path).unwrap();
    // check if the argument's essentiality changed correctly, and test reverse event
    let uninterpreted_fn = model.get_uninterpreted_fn(&f).unwrap();
    assert_eq!(uninterpreted_fn.get_essential(1), &Essentiality::True);
    let reverse_at_path = ["uninterpreted_fn", f.as_str(), "set_essentiality"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
/// Test changing uninterpreted function's expression via event.
fn test_change_fn_expression() {
    let mut model = ModelState::new_empty();
    let f = model.generate_uninterpreted_fn_id("f");
    model.add_empty_uninterpreted_fn(f.clone(), "f", 2).unwrap();
    let model_orig = model.clone();

    // test event for changing uninterpreted fn's expression
    let expression = "var0 | var1";
    let at_path = ["uninterpreted_fn", f.as_str(), "set_expression"];
    let event = mk_model_event(&at_path, Some(&expression));
    let result = model.perform_event(&event, &at_path).unwrap();

    // check if the expression changed correctly, and test reverse event
    let uninterpreted_fn = model.get_uninterpreted_fn(&f).unwrap();
    assert_eq!(uninterpreted_fn.get_fn_expression(), expression);
    let reverse_at_path = ["uninterpreted_fn", f.as_str(), "set_expression"];
    check_reverse(&mut model, &model_orig, result, &reverse_at_path);
}

#[test]
/// Test all kinds of refresh events.
fn test_refresh() {
    // first set the model (2 variables, 2 regulations, 2 functions)
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let regulations = vec!["a -> a", "b -| a", "b -| b"];
    let functions = vec![("f", "f", 2), ("g", "g", 3)];
    let expressions = vec!["a & !b", "!b"];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    model.add_multiple_uninterpreted_fns(functions).unwrap();
    model.add_multiple_regulations(regulations).unwrap();

    let var_a = model.get_var_id("a").unwrap();
    let var_b = model.get_var_id("b").unwrap();
    let fn_f = model.get_uninterpreted_fn_id("f").unwrap();
    let layout_id = ModelState::get_default_layout_id().to_string();
    model.set_update_fn(&var_a, expressions[0]).unwrap();
    model.set_update_fn(&var_b, expressions[1]).unwrap();

    // test variables getter
    let full_path = stringify_path(&["sketch", "model", "get_variables"]);
    let event = model.refresh(&full_path, &["get_variables"]).unwrap();
    let var_list: Vec<VariableData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(var_list.len(), 2);
    assert_eq!(var_list[0].name, "a_name".to_string());
    assert_eq!(var_list[0].id, var_a.to_string());
    assert_eq!(var_list[0].update_fn, expressions[0]);

    // test regulations getter
    let full_path = stringify_path(&["sketch", "model", "get_regulations"]);
    let event = model.refresh(&full_path, &["get_regulations"]).unwrap();
    let reg_list: Vec<RegulationData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(reg_list.len(), 3);
    assert_eq!(reg_list[0].target, var_a.to_string());
    assert_eq!(reg_list[0].regulator, var_a.to_string());

    // test layouts getter
    let full_path = stringify_path(&["sketch", "model", "get_layouts"]);
    let event = model.refresh(&full_path, &["get_layouts"]).unwrap();
    let layout_list: Vec<LayoutData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(layout_list.len(), 1);
    assert_eq!(layout_list[0].id, layout_id.clone());

    // test layout nodes getter
    let full_path = stringify_path(&["sketch", "model", "get_layout_nodes", &layout_id]);
    let event = model
        .refresh(&full_path, &["get_layout_nodes", &layout_id])
        .unwrap();
    let node_list: Vec<LayoutNodeData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(node_list.len(), 2);
    assert_eq!(node_list[0].variable, var_a.to_string());

    // test uninterpreted functions getter
    let full_path = stringify_path(&["sketch", "model", "get_uninterpreted_fns"]);
    let event = model
        .refresh(&full_path, &["get_uninterpreted_fns"])
        .unwrap();
    let uninterpreted_fn_list: Vec<UninterpretedFnData> =
        serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(uninterpreted_fn_list.len(), 2);
    assert_eq!(uninterpreted_fn_list[0].id, fn_f.to_string());

    // test whole model getter
    let full_path = stringify_path(&["sketch", "model", "get_whole_model"]);
    let event = model.refresh(&full_path, &["get_whole_model"]).unwrap();
    let model_data: ModelData = ModelData::from_json_str(&event.payload.unwrap()).unwrap();
    assert_eq!(model_data.uninterpreted_fns, uninterpreted_fn_list);
    assert_eq!(model_data.variables, var_list);
    assert_eq!(model_data.regulations, reg_list);
    assert_eq!(model_data.layouts, layout_list);
}
