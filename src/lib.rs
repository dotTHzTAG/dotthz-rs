//! Crate to load and save dotThz files in rust.
#![allow(dead_code)]
#![deny(missing_docs)]
#![deny(warnings)]

mod dotthz;
pub use dotthz::{DotthzFile, DotthzMeasurement, DotthzMetaData};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::path::PathBuf;
    use ndarray::array;
    use indexmap::IndexMap;
    use dotthz::{DotthzFile, DotthzMeasurement, DotthzMetaData};

    #[test]
    fn test_copy_and_compare_dotthz_files() -> Result<(), Box<dyn std::error::Error>> {
        for path in ["test_files/PVDF_520um.thz", "test_files/2_VariableTemperature.thz"] {

            // Path to an existing test HDF5 file (replace this with an actual test file path)
            let original_file_path = PathBuf::from(path);

            // Load data from the original file
            let original_dotthz = DotthzFile::load(original_file_path.clone())?;

            // Create a temporary file to save the copy
            let temp_file = NamedTempFile::new()?;
            let copy_file_path = temp_file.path().to_path_buf();

            // Save the data to the new temporary file
            original_dotthz.save(copy_file_path.clone())?;

            // Load data from the new copy file
            let copied_dotthz = DotthzFile::load(copy_file_path)?;

            // Compare the original and copied Dotthz structures
            assert_eq!(original_dotthz.groups.len(), copied_dotthz.groups.len());

            for (group_name, original_measurement) in &original_dotthz.groups {
                let copied_measurement = copied_dotthz.groups.get(group_name).expect("Group not found");

                // Compare metadata
                assert_eq!(original_measurement.meta_data.user, copied_measurement.meta_data.user);
                assert_eq!(original_measurement.meta_data.email, copied_measurement.meta_data.email);
                assert_eq!(original_measurement.meta_data.orcid, copied_measurement.meta_data.orcid);
                assert_eq!(original_measurement.meta_data.institution, copied_measurement.meta_data.institution);
                assert_eq!(original_measurement.meta_data.description, copied_measurement.meta_data.description);
                assert_eq!(original_measurement.meta_data.version, copied_measurement.meta_data.version);
                assert_eq!(original_measurement.meta_data.mode, copied_measurement.meta_data.mode);
                assert_eq!(original_measurement.meta_data.instrument, copied_measurement.meta_data.instrument);
                assert_eq!(original_measurement.meta_data.time, copied_measurement.meta_data.time);
                assert_eq!(original_measurement.meta_data.date, copied_measurement.meta_data.date);

                // Compare the metadata's key-value pairs
                assert_eq!(original_measurement.meta_data.md.len(), copied_measurement.meta_data.md.len());
                for (key, original_value) in &original_measurement.meta_data.md {
                    let copied_value = copied_measurement.meta_data.md.get(key).expect("Metadata key not found");
                    assert_eq!(original_value, copied_value);
                }

                // Compare datasets
                assert_eq!(original_measurement.datasets.len(), copied_measurement.datasets.len());
                for (dataset_name, original_dataset) in &original_measurement.datasets {
                    let copied_dataset = copied_measurement.datasets.get(dataset_name).expect("Dataset not found");
                    assert_eq!(original_dataset, copied_dataset);
                }
            }
        }

        Ok(())
    }

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
        file_to_write.save(path.clone())?;

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
