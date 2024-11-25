use hdf5::types::VarLenUnicode;
use hdf5::{Dataset, File, Group, H5Type};
use indexmap::IndexMap;
use ndarray::ArrayView;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;
use std::str::FromStr;

/// Metadata associated with a dotThz measurement.
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DotthzMetaData {
    /// The user responsible for the measurement.
    pub user: String,

    /// The email of the user.
    pub email: String,

    /// The ORCID identifier for the user.
    pub orcid: String,

    /// The institution of the user.
    pub institution: String,

    /// The description of the measurement.
    pub description: String,

    /// Additional metadata stored as key-value pairs.
    pub md: IndexMap<String, String>,

    /// dsDescription stored as key-value pairs.
    pub ds_description: Vec<String>,

    /// dotThz version
    pub version: String,

    /// The mode of measurement.
    pub mode: String,

    /// The instrument used for measurement.
    pub instrument: String,

    /// The time of measurement.
    pub time: String,

    /// The date of measurement.
    pub date: String,
}

/// A structure representing a .thz file according to the dotThz standard
pub struct DotthzFile {
    /// contains the Group and Dataset names
    file: File, // Keep a reference to the underlying HDF5 file
}

impl DotthzFile {
    /// Create an empty `DotthzFile` to the specified path.
    pub fn create(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        // Create a new HDF5 file at the specified path
        let file = File::create(path)?;
        Ok(Self { file })
    }

    /// Loads a `DotthzFile` from the specified path.
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        Ok(DotthzFile { file })
    }

    /// get group names
    pub fn get_group_names(&self) -> hdf5::Result<Vec<String>> {
        Ok(self
            .file
            .groups()?
            .iter()
            .map(|s| s.name())
            .collect::<Vec<String>>())
    }

    /// get group by name
    pub fn get_group(&self, group_name: &str) -> hdf5::Result<Group> {
        self.file.group(group_name)
    }

    /// get groups
    pub fn get_groups(&self) -> hdf5::Result<Vec<Group>> {
        self.file.groups()
    }

    /// get dataset names for a given group name
    pub fn get_dataset_names(&self, group_name: &str) -> hdf5::Result<Vec<String>> {
        Ok(self
            .file
            .group(group_name)?
            .datasets()?
            .iter()
            .map(|d| d.name())
            .collect::<Vec<String>>())
    }

    /// get dataset for a given group name by dataset name
    pub fn get_dataset(&self, group_name: &str, dataset_name: &str) -> hdf5::Result<Dataset> {
        self.file.group(group_name)?.dataset(dataset_name)
    }

    /// get datasets for a given group name
    pub fn get_datasets(&self, group_name: &str) -> hdf5::Result<Vec<Dataset>> {
        self.file.group(group_name)?.datasets()
    }

    /// set meta-data for a given group
    pub fn set_meta_data(
        &self,
        group: &mut Group,
        meta_data: &DotthzMetaData,
    ) -> Result<(), Box<dyn Error>> {
        // Save metadata attributes
        group
            .new_attr::<VarLenUnicode>()
            .create("description")?
            .write_scalar(&VarLenUnicode::from_str(&meta_data.description)?)?;

        group
            .new_attr::<VarLenUnicode>()
            .create("date")?
            .write_scalar(&VarLenUnicode::from_str(&meta_data.date)?)?;

        group
            .new_attr::<VarLenUnicode>()
            .create("instrument")?
            .write_scalar(&VarLenUnicode::from_str(&meta_data.instrument)?)?;

        group
            .new_attr::<VarLenUnicode>()
            .create("mode")?
            .write_scalar(&VarLenUnicode::from_str(&meta_data.mode)?)?;

        group
            .new_attr::<VarLenUnicode>()
            .create("thzVer")?
            .write_scalar(&VarLenUnicode::from_str(&meta_data.version)?)?;

        group
            .new_attr::<VarLenUnicode>()
            .create("time")?
            .write_scalar(&VarLenUnicode::from_str(&meta_data.time)?)?;

        let entry = VarLenUnicode::from_str(
            format!(
                "{}/{}/{}/{}",
                meta_data.orcid, meta_data.user, meta_data.email, meta_data.institution
            )
            .as_str(),
        )
        .unwrap();

        let attr = group.new_attr::<VarLenUnicode>().create("user")?;
        attr.write_scalar(&entry)?;

        // Save additional metadata
        let md_descriptions = meta_data
            .md
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");
        group
            .new_attr::<VarLenUnicode>()
            .shape(1)
            .create("mdDescription")?
            .write(&[VarLenUnicode::from_str(&md_descriptions)?])?;

        for (i, (_key, value)) in meta_data.md.iter().enumerate() {
            group
                .new_attr::<VarLenUnicode>()
                .create(format!("md{}", i + 1).as_str())?
                .write_scalar(&VarLenUnicode::from_str(value)?)?;
        }

        // Save dsDescription
        let ds_descriptions = meta_data.ds_description.join(", ");
        group
            .new_attr::<VarLenUnicode>()
            .shape(1)
            .create("dsDescription")?
            .write(&[VarLenUnicode::from_str(&ds_descriptions)?])?;
        Ok(())
    }

    /// extract meta-data for a given group by group name
    pub fn get_meta_data(&self, group_name: &str) -> hdf5::Result<DotthzMetaData> {
        let mut meta_data = DotthzMetaData::default();

        if let Ok(instrument) = self
            .file
            .group(group_name)?
            .attr("instrument")
            .and_then(|a| a.read_raw::<VarLenUnicode>())
        {
            if let Some(d) = instrument.first() {
                meta_data.instrument = d.to_string();
            }
        }

        // Load dataset descriptions
        if let Ok(ds_description) = self
            .file
            .group(group_name)?
            .attr("dsDescription")
            .and_then(|a| a.read_raw::<VarLenUnicode>())
        {
            let descriptions: Vec<String> = ds_description
                .iter()
                .flat_map(|s| s.split(", ").map(String::from).collect::<Vec<String>>())
                .collect();
            meta_data.ds_description = descriptions;
        }

        if let Ok(md_description) = self
            .file
            .group(group_name)?
            .attr("mdDescription")
            .and_then(|a| a.read_raw::<VarLenUnicode>())
        {
            // Convert ds_description to a vector of strings, splitting any single entry by ", "
            let descriptions: Vec<String> = if md_description.len() == 1 {
                // If there's only one entry, split it by ", "
                md_description[0]
                    .split(", ")
                    .map(|s| s.to_string())
                    .collect()
            } else {
                // Otherwise, assume it's already in the correct format
                md_description.iter().map(|s| s.to_string()).collect()
            };

            for (i, description) in descriptions.iter().enumerate() {
                // now read the mds
                if let Ok(md) = self
                    .file
                    .group(group_name)?
                    .attr(format!("md{}", i + 1).as_str())
                    .and_then(|a| a.read_raw::<f32>())
                {
                    if let Some(md) = md.first() {
                        meta_data
                            .md
                            .insert(description.to_string(), format!("{}", md));
                    }
                }
                if let Ok(md) = self
                    .file
                    .group(group_name)?
                    .attr(format!("md{}", i + 1).as_str())
                    .and_then(|a| a.read_raw::<VarLenUnicode>())
                {
                    if let Some(md) = md.first() {
                        meta_data
                            .md
                            .insert(description.to_string(), format!("{}", md));
                    }
                }
            }
        }

        if let Ok(mode) = self
            .file
            .group(group_name)?
            .attr("mode")
            .and_then(|a| a.read_raw::<VarLenUnicode>())
        {
            if let Some(d) = mode.first() {
                meta_data.mode = d.to_string();
            }
        }

        if let Ok(version) = self
            .file
            .group(group_name)?
            .attr("thzVer")
            .and_then(|a| a.read_raw::<VarLenUnicode>())
        {
            if let Some(d) = version.first() {
                meta_data.version = d.to_string();
            }
        }

        if let Ok(time) = self
            .file
            .group(group_name)?
            .attr("time")
            .and_then(|a| a.read_raw::<VarLenUnicode>())
        {
            if let Some(d) = time.first() {
                meta_data.time = d.to_string();
            }
        }

        if let Ok(date) = self
            .file
            .group(group_name)?
            .attr("date")
            .and_then(|a| a.read_raw::<VarLenUnicode>())
        {
            if let Some(d) = date.first() {
                meta_data.date = d.to_string();
            }
        }

        if let Ok(user_info) = self
            .file
            .group(group_name)?
            .attr("user")
            .and_then(|a| a.read_raw::<VarLenUnicode>())
        {
            if let Some(d) = user_info.first() {
                let user_info_str = d.to_string();
                let user_parts: Vec<&str> = user_info_str.split('/').collect();

                // Check each part individually to handle cases where fewer than 4 parts are available
                if let Some(part) = user_parts.first() {
                    meta_data.orcid = part.trim().into();
                }
                if let Some(part) = user_parts.get(1) {
                    meta_data.user = part.trim().into();
                }
                if let Some(part) = user_parts.get(2) {
                    meta_data.email = part.trim().into();
                }
                if let Some(part) = user_parts.get(3) {
                    meta_data.institution = part.trim().into();
                }
            }
        }
        Ok(meta_data)
    }

    /// Add a group with meta-data and group name to the `DotthzFile`.
    pub fn add_group(
        &mut self,
        group_name: &str,
        metadata: &DotthzMetaData,
    ) -> Result<Group, Box<dyn Error>> {
        let mut group = self.file.create_group(group_name)?;
        self.set_meta_data(&mut group, metadata)?;
        Ok(group)
    }

    /// Add a dataset to a given group by group name and dataset name.
    pub fn add_dataset<T, D>(
        &mut self,
        group_name: &str,
        dataset_name: &str,
        dataset: ArrayView<'_, T, D>,
    ) -> Result<(), Box<dyn Error>>
    where
        T: H5Type + Debug,
        D: ndarray::Dimension, // Ensure dimensions are compatible with HDF5
    {
        // Retrieve or create the group
        let group = self.file.group(group_name)?;
        // Create the dataset in the specified group with the shape from the ndarray
        let ds = group
            .new_dataset::<T>()
            .shape(dataset.shape())
            .create(dataset_name)?;

        // Write the data into the dataset
        ds.write(dataset)?;

        Ok(())
    }
}
