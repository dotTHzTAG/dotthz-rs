mod dotthz;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::path::PathBuf;
    use ndarray::array;
    use indexmap::IndexMap;
    use dotthz::{DotthzFile, DotthzMeasurement, DotthzMetaData}; // Adjust as needed to import your struct

    #[test]
    fn test_dotthz_save_and_load() -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary file to act as virtual storage
        let temp_file = NamedTempFile::new()?;
        let path: PathBuf = temp_file.path().to_path_buf();

        // Initialize test data for Dotthz
        let mut datasets = IndexMap::new();
        datasets.insert("ds1".to_string(), array![[1.0, 2.0], [3.0, 4.0]]);
        let meta_data = DotthzMetaData {
            user: "Test User".to_string(),
            email: "test@example.com".to_string(),
            orcid: "0000-0001-2345-6789".to_string(),
            institution: "Test Institute".to_string(),
            description: "Test description".to_string(),
            md: [("md1".to_string(), "Thickness (mm)".to_string())].into_iter().collect(),
            version: "1.0".to_string(),
            mode: "Test mode".to_string(),
            instrument: "Test instrument".to_string(),
            time: "12:34:56".to_string(),
            date: "2024-11-08".to_string(),
        };

        let mut groups = IndexMap::new();
        groups.insert(
            "group1".to_string(),
            DotthzMeasurement {
                datasets,
                meta_data,
            },
        );

        let file_to_write = DotthzFile { groups };
        
        // Save to the temporary file
        file_to_write.save_file(path.clone())?;

        // Load from the temporary file
        let loaded_file = DotthzFile::load(path)?;

        // Compare original and loaded data
        assert_eq!(file_to_write.groups.len(), loaded_file.groups.len());

        for (group_name, measurement) in &file_to_write.groups {
            let loaded_measurement = loaded_file.groups.get(group_name).expect("Group not found");

            // Compare metadata
            assert_eq!(measurement.meta_data.user, loaded_measurement.meta_data.user);
            assert_eq!(measurement.meta_data.email, loaded_measurement.meta_data.email);
            assert_eq!(measurement.meta_data.orcid, loaded_measurement.meta_data.orcid);
            assert_eq!(measurement.meta_data.institution, loaded_measurement.meta_data.institution);
            assert_eq!(measurement.meta_data.description, loaded_measurement.meta_data.description);
            assert_eq!(measurement.meta_data.version, loaded_measurement.meta_data.version);
            assert_eq!(measurement.meta_data.mode, loaded_measurement.meta_data.mode);
            assert_eq!(measurement.meta_data.instrument, loaded_measurement.meta_data.instrument);
            assert_eq!(measurement.meta_data.time, loaded_measurement.meta_data.time);
            assert_eq!(measurement.meta_data.date, loaded_measurement.meta_data.date);

            // Compare mds
            assert_eq!(measurement.meta_data.md.len(), loaded_measurement.meta_data.md.len());
            for (dataset_name, dataset) in &measurement.meta_data.md {
                let loaded_dataset = loaded_measurement.meta_data.md.get(dataset_name).expect("Md not found");
                assert_eq!(dataset, loaded_dataset);
            }
            
            // Compare datasets
            assert_eq!(measurement.datasets.len(), loaded_measurement.datasets.len());
            for (dataset_name, dataset) in &measurement.datasets {
                let loaded_dataset = loaded_measurement.datasets.get(dataset_name).expect("Dataset not found");
                assert_eq!(dataset, loaded_dataset);
            }
        }

        Ok(())
    }
}
