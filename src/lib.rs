//! Crate to load and save dotThz files in Rust.
#![allow(dead_code)]
#![deny(missing_docs)]
#![deny(warnings)]

mod dotthz;
pub use dotthz::{DotthzFile, DotthzMetaData};

#[cfg(test)]
mod tests {
    use super::*;
    use dotthz::{DotthzFile, DotthzMetaData};
    use hdf5::Dataset;
    use ndarray::{array, Array2};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn assert_datasets_equal(
        ds1: &Dataset,
        ds2: &Dataset,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data1: Vec<f32> = ds1.read_raw()?;
        let data2: Vec<f32> = ds2.read_raw()?;
        assert_eq!(data1, data2, "Dataset contents differ");

        let shape1 = ds1.shape();
        let shape2 = ds2.shape();
        assert_eq!(shape1, shape2, "Dataset shapes differ");

        Ok(())
    }

    #[test]
    fn test_copy_and_compare_dotthz_files() -> Result<(), Box<dyn std::error::Error>> {
        for path in [
            "test_files/PVDF_520um.thz",
            "test_files/2_VariableTemperature.thz",
            //   "test_files/image.thz",
        ] {
            // Path to an existing test HDF5 file (replace this with an actual test file path)
            let original_file_path = PathBuf::from(path);

            // Load data from the original file
            let original_dotthz = DotthzFile::open(&original_file_path)?;

            // Create a temporary file to save the copy
            let temp_file = NamedTempFile::new()?;
            let copy_file_path = temp_file.path().to_path_buf();

            // Create the new temporary file
            let mut output_dotthz = DotthzFile::create(&copy_file_path)?;

            // Save the metadata to the new temporary file
            let group_names = original_dotthz.get_group_names()?;
            for group_name in group_names.iter() {
                let group = original_dotthz.get_group(group_name)?;
                let meta_data = original_dotthz.get_meta_data(&group.name())?;
                output_dotthz.add_group(group_name, &meta_data)?;
            }

            // Save the data to the new temporary file
            let group_names = output_dotthz.get_group_names()?;
            for group_name in group_names.iter() {
                for dataset_name in original_dotthz.get_dataset_names(group_name)? {
                    let dataset = original_dotthz.get_dataset(group_name, &dataset_name)?;
                    let data = dataset.read_dyn::<f32>()?;
                    output_dotthz.add_dataset(group_name, &dataset_name, data.view())?;
                }
            }

            // Load data from the new copy file
            let copied_dotthz = DotthzFile::open(&copy_file_path)?;

            for old_group in original_dotthz.get_groups()? {
                let new_group = copied_dotthz.get_group(&old_group.name())?;
                // Compare the original and copied metadata
                assert_eq!(
                    original_dotthz.get_meta_data(&old_group.name())?,
                    copied_dotthz.get_meta_data(&new_group.name())?
                );
            }

            let group_names = output_dotthz.get_group_names()?;
            for group_name in group_names.iter() {
                for dataset_name in output_dotthz.get_dataset_names(group_name)? {
                    let old_dataset = original_dotthz.get_dataset(group_name, &dataset_name)?;
                    let new_dataset = copied_dotthz.get_dataset(group_name, &dataset_name)?;

                    // Compare the original and copied datasets
                    assert_eq!(old_dataset.shape(), new_dataset.shape());
                    assert_datasets_equal(&old_dataset, &new_dataset)?;
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

        // Initialize test metadata and data
        let meta_data = DotthzMetaData {
            user: "Test User".to_string(),
            email: "test@example.com".to_string(),
            orcid: "0000-0001-2345-6789".to_string(),
            institution: "Test Institute".to_string(),
            description: "Test description".to_string(),
            md: [("Thickness (mm)".to_string(), "0.52".to_string())]
                .into_iter()
                .collect(),
            ds_description: vec!["ds1".to_string()],
            version: "1.0".to_string(),
            mode: "Test mode".to_string(),
            instrument: "Test instrument".to_string(),
            time: "12:34:56".to_string(),
            date: "2024-11-08".to_string(),
        };

        // Initialize a dataset
        let mut original_dotthz = DotthzFile::create(&path)?;
        let group_name = "Measurement".to_string();
        original_dotthz.add_group(&group_name, &meta_data)?;

        let dataset_name = "test_dataset".to_string();
        let dataset_data: Array2<f32> = array![[1.0, 2.0], [3.0, 4.0], [3.0, 4.0]];

        original_dotthz.add_dataset(&group_name, &dataset_name, dataset_data.view())?;

        // Load data from the new copy file
        let copied_dotthz = DotthzFile::open(&path)?;

        for (old_group, new_group) in original_dotthz
            .get_groups()?
            .iter()
            .zip(copied_dotthz.get_groups()?.iter())
        {
            // Compare the original and copied metadata
            assert_eq!(
                original_dotthz.get_meta_data(&old_group.name())?,
                copied_dotthz.get_meta_data(&new_group.name())?
            );
        }

        let group_names = original_dotthz.get_group_names()?;
        for group_name in group_names.iter() {
            for dataset_name in original_dotthz.get_dataset_names(group_name)? {
                let old_dataset = original_dotthz.get_dataset(group_name, &dataset_name)?;
                let new_dataset = copied_dotthz.get_dataset(group_name, &dataset_name)?;

                // Compare the original and copied datasets
                assert_eq!(old_dataset.shape(), new_dataset.shape());
                assert_datasets_equal(&old_dataset, &new_dataset)?;
            }
        }

        Ok(())
    }
}
