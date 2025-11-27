use crate::sketchbook::observations::{Dataset, Observation, ObservationManager, VarValue};
use std::str::FromStr;

impl ObservationManager {
    /// Parse a dataset from a CSV string. The header line specifies variables, following lines
    /// represent individual observations (id and values).
    ///
    /// The resulting dataset has an empty annotation string (same for all its observations).
    ///
    /// For example, the following might be a valid CSV string for a dataset with 2 observations:
    ///    ID,YOX1,CLN3,YHP1,ACE2,SWI5,MBF
    ///    Observation1,0,1,0,1,0,1
    ///    Observation2,1,0,*,1,0,*
    ///
    pub fn parse_dataset_from_csv(name: &str, csv_content: &str) -> Result<Dataset, String> {
        let mut rdr = csv::Reader::from_reader(csv_content.as_bytes());

        // parse variable names from the header (and strip whitespaces)
        let header = rdr.headers().map_err(|e| e.to_string())?.clone();
        let variables = header.iter().skip(1).map(|s| s.trim()).collect();

        // parse all rows as observations
        let mut observations = Vec::new();
        for result in rdr.records() {
            let record = result.map_err(|e| e.to_string())?;
            if record.is_empty() {
                return Err("Cannot import empty observation.".to_string());
            }
            let id: &str = record.get(0).unwrap().trim(); // trim the ID
            let values: Vec<VarValue> = record
                .iter()
                .skip(1)
                .map(|s| VarValue::from_str(s.trim()))
                .collect::<Result<Vec<VarValue>, String>>()?;
            let observation = Observation::new(values, id)?;
            observations.push(observation);
        }
        Dataset::new(name, observations, variables)
    }

    /// Load a dataset from a given CSV file. Reads the file into a string and then parses it
    /// into a dataset using [Self::parse_dataset_from_csv].
    pub fn load_dataset(name: &str, csv_path: &str) -> Result<Dataset, String> {
        let csv_content = std::fs::read_to_string(csv_path).map_err(|e| e.to_string())?;
        Self::parse_dataset_from_csv(name, &csv_content)
    }

    /// Load a dataset from given CSV file, and add it to this `ObservationManager`.
    ///
    /// The header line specifies variables, following lines represent individual observations
    /// (id and values). See [Self::parse_dataset_from_csv] for details on the format.
    pub fn load_and_add_dataset(&mut self, csv_path: &str, id: &str) -> Result<(), String> {
        // use same name as ID
        let dataset = Self::load_dataset(id, csv_path)?;
        self.add_dataset_by_str(id, dataset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketchbook::observations::{Dataset, Observation};

    #[test]
    fn test_dataset_from_csv_string() {
        // Prepare expected dataset
        let obs1 = Observation::try_from_str("*11", "obs1").unwrap();
        let obs2 = Observation::try_from_str("000", "obs2").unwrap();
        let obs_list = vec![obs1, obs2];
        let var_names = vec!["a", "b", "c"];
        let dataset_name = "dataset_name";
        let expected_dataset = Dataset::new(dataset_name, obs_list, var_names).unwrap();

        // Parse the dataset from CSV string and compare the two
        // Include white spaces that should be handled automatically
        let csv_string = "ID , a , b , c \n obs1 , * , 1 , 1 \n obs2 , 0 , 0 , 0 \n";
        let parsed_dataset =
            ObservationManager::parse_dataset_from_csv(dataset_name, csv_string).unwrap();
        assert_eq!(parsed_dataset, expected_dataset);
    }
}
