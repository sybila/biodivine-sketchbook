use crate::sketchbook::data_structs::{DatasetData, DynPropertyData, ModelData, StatPropertyData};
use crate::sketchbook::model::ModelState;
use crate::sketchbook::observations::ObservationManager;
use crate::sketchbook::properties::PropertyManager;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Structure for sending/exporting data about the whole Sketch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SketchData {
    pub model: ModelData,
    pub datasets: Vec<DatasetData>,
    pub dyn_properties: Vec<DynPropertyData>,
    pub stat_properties: Vec<StatPropertyData>,
}

impl<'de> JsonSerde<'de> for SketchData {}

impl SketchData {
    /// Create new `SketchData` instance given a reference to all necessary manager classes.
    pub fn new(
        model: &ModelState,
        observations: &ObservationManager,
        properties: &PropertyManager,
    ) -> SketchData {
        let datasets = observations
            .datasets()
            .map(|(d_id, d)| DatasetData::from_dataset(d_id, d))
            .collect();
        let dyn_properties = properties
            .dyn_props()
            .map(|(p_id, p)| DynPropertyData::from_property(p_id, p))
            .collect();
        let stat_properties = properties
            .stat_props()
            .map(|(p_id, _)| StatPropertyData::new(p_id.as_str()))
            .collect();

        SketchData {
            model: ModelData::new(model),
            datasets,
            dyn_properties,
            stat_properties,
        }
    }
}
