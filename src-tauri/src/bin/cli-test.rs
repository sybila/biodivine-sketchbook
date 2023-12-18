use aeon_sketchbook::sketchbook::simplified_structs::RegulationData;
use aeon_sketchbook::sketchbook::{ModelState, Regulation};
use biodivine_lib_param_bn::RegulatoryGraph;

fn main() {
    let mut r = RegulatoryGraph::new(vec!["a".to_string(), "b".to_string()]);
    r.add_string_regulation("a -> b").unwrap();
    r.add_string_regulation("b -| a").unwrap();

    let reg_state = ModelState::from_reg_graph(r);
    match reg_state {
        Ok(r) => println!("{}", r),
        Err(e) => println!("{}", e),
    }

    let reg = Regulation::try_from_string("a->b").unwrap();
    let regulation_data = RegulationData::from_reg(&reg);
    println!("{}", serde_json::to_string(&regulation_data).unwrap());
}
