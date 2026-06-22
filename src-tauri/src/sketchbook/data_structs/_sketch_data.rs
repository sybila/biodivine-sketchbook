use crate::sketchbook::data_structs::{
    DatasetData, DynPropertyData, ModelData, PerturbationData, StatPropertyData,
};
use crate::sketchbook::model::ModelState;
use crate::sketchbook::observations::ObservationManager;
use crate::sketchbook::perturbations::PerturbationManager;
use crate::sketchbook::properties::PropertyManager;
use crate::sketchbook::{JsonSerde, Sketch};
use serde::{Deserialize, Serialize};

/// Structure for sending/exporting data about the whole Sketch.
///
/// When importing data, the `SketchData` structure is used to create a new `Sketch` instance.
/// The structure is serialized and deserialized using [serde] and [serde_json].
/// All fields apart from `model` are optional and will be initialized with default values if not provided.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SketchData {
    pub model: ModelData,
    #[serde(default)]
    pub datasets: Vec<DatasetData>,
    #[serde(default)]
    pub dyn_properties: Vec<DynPropertyData>,
    #[serde(default)]
    pub stat_properties: Vec<StatPropertyData>,
    #[serde(default)]
    pub perturbations: Vec<PerturbationData>,
    #[serde(default)]
    pub annotation: String,
}

impl JsonSerde<'_> for SketchData {}

impl SketchData {
    /// Create new `SketchData` instance given a reference to all necessary manager classes.
    pub fn new(
        model: &ModelState,
        observations: &ObservationManager,
        properties: &PropertyManager,
        perturbations: &PerturbationManager,
        annotation: &str,
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
            .map(|(p_id, p)| StatPropertyData::from_property(p_id, p))
            .collect();
        let perturbations = perturbations
            .perturbations_iter()
            .map(|(p_id, p)| PerturbationData::from_perturbation(p_id, p))
            .collect();

        SketchData {
            model: ModelData::from_model(model),
            datasets,
            dyn_properties,
            stat_properties,
            perturbations,
            annotation: annotation.to_string(),
        }
    }

    /// Create new `SketchData` instance given a reference to the `Sketch` instance.
    pub fn new_from_sketch(sketch: &Sketch) -> SketchData {
        Self::new(
            &sketch.model,
            &sketch.observations,
            &sketch.properties,
            &sketch.perturbations,
            &sketch.annotation,
        )
    }
}
