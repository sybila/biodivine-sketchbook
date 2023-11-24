use aeon_sketchbook::sketchbook::RegulationsState;
use biodivine_lib_param_bn::RegulatoryGraph;

fn main() {
    let mut r = RegulatoryGraph::new(vec!["a".to_string(), "b".to_string()]);
    r.add_string_regulation("a -> b").unwrap();
    r.add_string_regulation("b -| a").unwrap();

    let reg_state = RegulationsState::from_reg_graph(r);
    match reg_state {
        Ok(r) => println!("{}", r),
        Err(e) => println!("{}", e),
    }
}
