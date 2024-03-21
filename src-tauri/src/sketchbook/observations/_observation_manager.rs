use crate::sketchbook::observations::{Dataset, DatasetIterator};
use crate::sketchbook::DatasetId;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct ObservationManager {
    datasets: HashMap<DatasetId, Dataset>,
}

impl Default for ObservationManager {
    /// Default manager instance with no datasets.
    fn default() -> ObservationManager {
        ObservationManager::new_empty()
    }
}

/// Creating instances of `ObservationManager`.
impl ObservationManager {
    /// Instantiate `ObservationManager` with empty list of datasets.
    pub fn new_empty() -> ObservationManager {
        ObservationManager {
            datasets: HashMap::new(),
        }
    }

    /// Instantiate `ObservationManager` with given list of ID-dataset pairs.
    pub fn new_from_datasets(datasets: Vec<(&str, Dataset)>) -> Result<ObservationManager, String> {
        let mut manager = ObservationManager::new_empty();

        let prop_id_set = datasets.iter().map(|pair| pair.0).collect::<HashSet<_>>();
        if prop_id_set.len() != datasets.len() {
            return Err(format!("Datasets {:?} contain duplicate IDs.", datasets));
        }

        for (id, dataset) in datasets {
            let dataset_id = DatasetId::new(id)?;
            manager.datasets.insert(dataset_id, dataset);
        }
        Ok(manager)
    }
}

/// Editing `ObservationManager`.
impl ObservationManager {
    /// Add a new dataset with given `id` to this `ObservationManager`.
    ///
    /// The ID must be valid identifier that is not already used by some other dataset.
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_dataset(&mut self, id: DatasetId, dataset: Dataset) -> Result<(), String> {
        self.assert_no_dataset(&id)?;
        self.datasets.insert(id, dataset);
        Ok(())
    }

    /// Add a new dataset with given string `id` to this `ObservationManager`.
    ///
    /// The ID must be valid identifier that is not already used by some other dataset.
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_dataset_by_str(&mut self, id: &str, dataset: Dataset) -> Result<(), String> {
        let dataset_id = DatasetId::new(id)?;
        self.add_dataset(dataset_id, dataset)
    }

    /// Shorthand to add a list of new datasets with given string IDs to this manager.
    ///
    /// The ID must be valid identifier that is not already used by some other dataset.
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_multiple_datasets(
        &mut self,
        id_name_pairs: Vec<(&str, Dataset)>,
    ) -> Result<(), String> {
        // before making any changes, check if all IDs are actually valid
        for (id, _) in &id_name_pairs {
            let dataset_id = DatasetId::new(id)?;
            self.assert_no_dataset(&dataset_id)?;
        }
        for (id, name) in id_name_pairs {
            self.add_dataset_by_str(id, name)?;
        }
        Ok(())
    }

    /// Swap content of a dataset with given `id`. The ID must be valid identifier.
    pub fn swap_dataset_content(
        &mut self,
        id: &DatasetId,
        new_content: Dataset,
    ) -> Result<(), String> {
        self.assert_valid_dataset(id)?;
        self.datasets.insert(id.clone(), new_content);
        Ok(())
    }

    /// Set the id of dataset with `original_id` to `new_id`.
    pub fn set_dataset_id(
        &mut self,
        original_id: &DatasetId,
        new_id: DatasetId,
    ) -> Result<(), String> {
        self.assert_valid_dataset(original_id)?;
        self.assert_no_dataset(&new_id)?;

        if let Some(dataset) = self.datasets.remove(original_id) {
            self.datasets.insert(new_id.clone(), dataset);
        } else {
            panic!("Error when modifying dataset's id in the dataset map.");
        }
        Ok(())
    }

    /// Set the id of dataset with `original_id` to `new_id`.
    pub fn set_dataset_id_by_str(&mut self, original_id: &str, new_id: &str) -> Result<(), String> {
        let original_id = DatasetId::new(original_id)?;
        let new_id = DatasetId::new(new_id)?;
        self.set_dataset_id(&original_id, new_id)
    }

    /// Remove the dataset with given `id` from this manager.
    /// Returns `Err` in case the `id` is not a valid dataset's identifier.
    pub fn remove_dataset(&mut self, id: &DatasetId) -> Result<(), String> {
        self.assert_valid_dataset(id)?;

        if self.datasets.remove(id).is_none() {
            panic!("Error when removing dataset {id} from the dataset map.")
        }
        Ok(())
    }

    /// Remove the dataset with given string `id` from this manager.
    /// Returns `Err` in case the `id` is not a valid dataset's identifier.
    pub fn remove_dataset_by_str(&mut self, id: &str) -> Result<(), String> {
        let dataset_id = DatasetId::new(id)?;
        self.remove_dataset(&dataset_id)
    }
}

/// Observing the `ObservationManager`.
impl ObservationManager {
    /// The number of datasets in this `ObservationManager`.
    pub fn num_datasets(&self) -> usize {
        self.datasets.len()
    }

    /// Check if there is a dataset with given Id.
    pub fn is_valid_dataset_id(&self, id: &DatasetId) -> bool {
        self.datasets.contains_key(id)
    }

    /// Return a valid dataset's `DatasetId` corresponding to the given str `id`.
    ///
    /// Return `Err` if such dataset does not exist (and the ID is invalid).
    pub fn get_dataset_id(&self, id: &str) -> Result<DatasetId, String> {
        let dataset_id = DatasetId::from_str(id)?;
        if self.is_valid_dataset_id(&dataset_id) {
            return Ok(dataset_id);
        }
        Err(format!("Dataset with ID {id} does not exist."))
    }

    /// Return a `Dataset` corresponding to a given `DatasetId`.
    ///
    /// Return `Err` if such dataset does not exist (the ID is invalid in this context).
    pub fn get_dataset(&self, id: &DatasetId) -> Result<&Dataset, String> {
        let dataset = self
            .datasets
            .get(id)
            .ok_or(format!("Dataset with ID {id} does not exist."))?;
        Ok(dataset)
    }

    /// Return an iterator over all datasets of this model.
    pub fn datasets(&self) -> DatasetIterator {
        self.datasets.iter()
    }

    /// **(internal)** Utility method to ensure there is no dataset with given ID yet.
    fn assert_no_dataset(&self, id: &DatasetId) -> Result<(), String> {
        if self.is_valid_dataset_id(id) {
            Err(format!("Dataset with id {id} already exists."))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a dataset with given ID.
    fn assert_valid_dataset(&self, id: &DatasetId) -> Result<(), String> {
        if self.is_valid_dataset_id(id) {
            Ok(())
        } else {
            Err(format!("Dataset with id {id} does not exist."))
        }
    }
}
