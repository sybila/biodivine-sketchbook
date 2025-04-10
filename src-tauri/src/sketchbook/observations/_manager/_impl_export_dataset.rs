use crate::sketchbook::ids::DatasetId;
use crate::sketchbook::observations::ObservationManager;
use std::fs::File;
use std::io::Write;

impl ObservationManager {
    /// Convert the dataset to a CSV string.
    ///
    /// See [Self::parse_dataset_from_csv] for the details of the format. The first line contains
    /// variable names as a header. Each subsequent line represents an observation (ID and values).
    fn dataset_to_csv_string(&self, dataset_id: &DatasetId) -> Result<String, String> {
        let dataset = self.get_dataset(dataset_id)?;
        let mut csv_string = String::new();

        // Add header line with variable names
        csv_string.push_str("ID");
        for var_id in dataset.variables() {
            csv_string.push(',');
            csv_string.push_str(var_id.as_str());
        }
        csv_string.push('\n');

        // Add each observation as a line
        for obs in dataset.observations() {
            csv_string.push_str(obs.get_id().as_str());
            for value in obs.get_values() {
                csv_string.push(',');
                csv_string.push_str(&value.to_string());
            }
            csv_string.push('\n');
        }

        Ok(csv_string)
    }

    /// Export particular dataset to a CSV file at given path.
    ///
    /// See [Self::parse_dataset_from_csv] for the details of the format. In short, the
    /// header line specifies variable IDs, and each subsequent line represents individual
    /// observations (id and values).
    pub fn export_dataset_to_csv(
        &self,
        dataset_id: &DatasetId,
        csv_path: &str,
    ) -> Result<(), String> {
        let csv_str = self.dataset_to_csv_string(dataset_id)?;

        let mut file = File::create(csv_path).map_err(|e| e.to_string())?;
        // write dataset in CSV to the file
        file.write_all(csv_str.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketchbook::observations::{Dataset, Observation};

    #[test]
    fn test_dataset_to_csv_string() {
        // Prepare a dataset with observations
        let obs1 = Observation::try_from_str("*11", "obs1").unwrap();
        let obs2 = Observation::try_from_str("000", "obs2").unwrap();
        let obs_list = vec![obs1, obs2];
        let var_names = vec!["a", "b", "c"];
        let dataset = Dataset::new("dataset_name", obs_list, var_names).unwrap();

        // Instantiate an ObservationManager instance
        let mut manager = ObservationManager::new_empty();
        let dataset_id = DatasetId::new("d").unwrap();
        manager.add_dataset(dataset_id.clone(), dataset).unwrap();

        // Convert dataset to CSV string and compare with expected output
        let csv_string = manager.dataset_to_csv_string(&dataset_id).unwrap();
        let expected_csv = "ID,a,b,c\nobs1,*,1,1\nobs2,0,0,0\n";
        assert_eq!(csv_string, expected_csv);
    }
}
