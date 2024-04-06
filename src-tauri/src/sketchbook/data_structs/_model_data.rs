use crate::sketchbook::data_structs::{
    LayoutData, RegulationData, UninterpretedFnData, VariableData,
};
use crate::sketchbook::model::ModelState;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Structure for sending/exporting all necessary data about the model part of the sketch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelData {
    pub variables: Vec<VariableData>,
    pub regulations: Vec<RegulationData>,
    pub uninterpreted_fns: Vec<UninterpretedFnData>,
    pub layouts: Vec<LayoutData>,
}

impl<'de> JsonSerde<'de> for ModelData {}

impl ModelData {
    /// Create new `SketchData` instance given a reference to a model manager instance.
    pub fn new(model: &ModelState) -> ModelData {
        let variables = model
            .variables()
            .map(|(id, v)| VariableData::from_var(id, v, model.get_update_fn(id).unwrap()))
            .collect();
        let regulations = model.regulations().map(RegulationData::from_reg).collect();
        let uninterpreted_fns = model
            .uninterpreted_fns()
            .map(|(id, f)| UninterpretedFnData::from_fn(id, f))
            .collect();
        let layouts = model
            .layouts()
            .map(|(id, l)| LayoutData::from_layout(id, l))
            .collect();

        ModelData {
            variables,
            regulations,
            uninterpreted_fns,
            layouts,
        }
    }
}
