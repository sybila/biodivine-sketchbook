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

impl JsonSerde<'_> for ModelData {}

impl ModelData {
    /// Create new `SketchData` instance given a reference to a model manager instance.
    pub fn from_model(model: &ModelState) -> ModelData {
        let mut variables: Vec<_> = model
            .variables()
            .map(|(id, v)| VariableData::from_var(id, v, model.get_update_fn(id).unwrap()))
            .collect();
        variables.sort_by(|a, b| a.id.cmp(&b.id));

        let mut regulations: Vec<_> = model.regulations().map(RegulationData::from_reg).collect();
        regulations.sort_by(|a, b| {
            let id_comparison = a.regulator.cmp(&b.regulator);
            if id_comparison == std::cmp::Ordering::Equal {
                a.target.cmp(&b.target)
            } else {
                id_comparison
            }
        });

        let mut uninterpreted_fns: Vec<_> = model
            .uninterpreted_fns()
            .map(|(id, f)| UninterpretedFnData::from_fn(id, f))
            .collect();
        uninterpreted_fns.sort_by(|a, b| a.id.cmp(&b.id));

        let mut layouts: Vec<_> = model
            .layouts()
            .map(|(id, l)| LayoutData::from_layout(id, l))
            .collect();
        layouts.sort_by(|a, b| a.id.cmp(&b.id));

        ModelData {
            variables,
            regulations,
            uninterpreted_fns,
            layouts,
        }
    }
}
