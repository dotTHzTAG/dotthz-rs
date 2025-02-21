use hdf5::file::{FileAccess, FileCreate};
use hdf5::types::VarLenUnicode;
use hdf5::{Dataset, File, Group, H5Type, OpenMode};
use indexmap::IndexMap;
use ndarray::ArrayView;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
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
    /// Create an empty `DotthzFile` to the specified path, truncates if exists.
    pub fn create(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        // Create a new HDF5 file at the specified path
        let file = File::create(path)?;
        Ok(Self { file })
    }

    /// Loads a `DotthzFile` from the specified path as read-only, file must exist.
    pub fn open(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let file = File::open(filename)?;
        Ok(DotthzFile { file })
    }

    /// Opens a file as read/write, file must exist.
    pub fn open_rw<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open_rw(filename)?;
        Ok(DotthzFile { file })
    }

    /// Creates a file, fails if exists.
    pub fn create_excl<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn Error>> {
        let file = File::create_excl(filename)?;
        Ok(DotthzFile { file })
    }

    /// Opens a file as read/write if exists, creates otherwise.
    pub fn append<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn Error>> {
        let file = File::append(filename)?;
        Ok(DotthzFile { file })
    }

    /// Opens a file in a given mode.
    pub fn open_as<P: AsRef<Path>>(filename: P, mode: OpenMode) -> Result<Self, Box<dyn Error>> {
        let file = File::open_as(filename, mode)?;
        Ok(DotthzFile { file })
    }

    /// Returns the file size in bytes (or 0 if the file handle is invalid).
    pub fn size(&self) -> u64 {
        self.file.size()
    }

    /// Returns the free space in the file in bytes (or 0 if the file handle is invalid).
    pub fn free_space(&self) -> u64 {
        self.file.free_space()
    }

    /// Returns true if the file was opened in a read-only mode.
    pub fn is_read_only(&self) -> bool {
        self.file.is_read_only()
    }

    /// Returns the userblock size in bytes (or 0 if the file handle is invalid).
    pub fn userblock(&self) -> u64 {
        self.file.userblock()
    }

    /// Flushes the file to the storage medium.
    pub fn flush(&self) -> Result<(), Box<dyn Error>> {
        self.file.flush()?;
        Ok(())
    }

    /// Closes the file and invalidates all open handles for contained objects.
    pub fn close(self) -> Result<(), Box<dyn Error>> {
        self.file.close()?;
        Ok(())
    }

    /// Returns a copy of the file access property list.
    pub fn access_plist(&self) -> hdf5::Result<FileAccess> {
        self.file.access_plist()
    }

    /// A short alias for `access_plist()`.
    pub fn fapl(&self) -> hdf5::Result<FileAccess> {
        self.file.access_plist()
    }
    /// Returns a copy of the file creation property list.
    pub fn create_plist(&self) -> hdf5::Result<FileCreate> {
        self.file.create_plist()
    }

    /// A short alias for `create_plist()`.
    pub fn fcpl(&self) -> hdf5::Result<FileCreate> {
        self.file.create_plist()
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
        // Save metadata attributes, create if they already exist
        if let Ok(attr) = group.attr("description") {
            attr.write_scalar(&VarLenUnicode::from_str(&meta_data.description)?)?;
        } else {
            group
                .new_attr::<VarLenUnicode>()
                .create("description")?
                .write_scalar(&VarLenUnicode::from_str(&meta_data.description)?)?;
        }

        if let Ok(attr) = group.attr("date") {
            attr.write_scalar(&VarLenUnicode::from_str(&meta_data.date)?)?;
        } else {
            group
                .new_attr::<VarLenUnicode>()
                .create("date")?
                .write_scalar(&VarLenUnicode::from_str(&meta_data.date)?)?;
        }

        if let Ok(attr) = group.attr("instrument") {
            attr.write_scalar(&VarLenUnicode::from_str(&meta_data.instrument)?)?;
        } else {
            group
                .new_attr::<VarLenUnicode>()
                .create("instrument")?
                .write_scalar(&VarLenUnicode::from_str(&meta_data.instrument)?)?;
        }

        if let Ok(attr) = group.attr("mode") {
            attr.write_scalar(&VarLenUnicode::from_str(&meta_data.mode)?)?;
        } else {
            group
                .new_attr::<VarLenUnicode>()
                .create("mode")?
                .write_scalar(&VarLenUnicode::from_str(&meta_data.mode)?)?;
        }

        if let Ok(attr) = group.attr("thzVer") {
            attr.write_scalar(&VarLenUnicode::from_str(&meta_data.version)?)?;
        } else {
            group
                .new_attr::<VarLenUnicode>()
                .create("thzVer")?
                .write_scalar(&VarLenUnicode::from_str(&meta_data.version)?)?;
        }

        if let Ok(attr) = group.attr("time") {
            attr.write_scalar(&VarLenUnicode::from_str(&meta_data.time)?)?;
        } else {
            group
                .new_attr::<VarLenUnicode>()
                .create("time")?
                .write_scalar(&VarLenUnicode::from_str(&meta_data.time)?)?;
        }

        let entry = VarLenUnicode::from_str(
            format!(
                "{}/{}/{}/{}",
                meta_data.orcid, meta_data.user, meta_data.email, meta_data.institution
            )
            .as_str(),
        )
        .unwrap();

        if let Ok(attr) = group.attr("user") {
            attr.write_scalar(&entry)?;
        } else {
            let attr = group.new_attr::<VarLenUnicode>().create("user")?;
            attr.write_scalar(&entry)?;
        }

        // Save additional metadata
        let md_descriptions = meta_data
            .md
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");

        if let Ok(attr) = group.attr("mdDescription") {
            attr.write_raw(&[VarLenUnicode::from_str(&md_descriptions)?])?;
        } else {
            group
                .new_attr::<VarLenUnicode>()
                .create("mdDescription")?
                .write_raw(&[VarLenUnicode::from_str(&md_descriptions)?])?;
        }

        for (i, (_key, value)) in meta_data.md.iter().enumerate() {
            if let Ok(attr) = group.attr(format!("md{}", i + 1).as_str()) {
                if let Ok(parsed) = value.parse::<f32>() {
                    attr.write_scalar(&parsed)?;
                } else {
                    attr.write_scalar(&VarLenUnicode::from_str(value)?)?;
                }
            } else {
                group
                    .new_attr::<VarLenUnicode>()
                    .create(format!("md{}", i + 1).as_str())?
                    .write_scalar(&VarLenUnicode::from_str(value)?)?;
            }
        }

        // Save dsDescription
        let ds_descriptions = meta_data.ds_description.join(", ");

        if let Ok(attr) = group.attr("dsDescription") {
            attr.write_raw(&[VarLenUnicode::from_str(&ds_descriptions)?])?;
        } else {
            group
                .new_attr::<VarLenUnicode>()
                .create("dsDescription")?
                .write_raw(&[VarLenUnicode::from_str(&ds_descriptions)?])?;
        }
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

        if let Ok(description) = self
            .file
            .group(group_name)?
            .attr("description")
            .and_then(|a| a.read_raw::<VarLenUnicode>())
        {
            if let Some(d) = description.first() {
                meta_data.description = d.to_string();
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

    /// remove a meta_data attribute

    pub fn remove_meta_data_attribute(
        &mut self,
        group_name: &str,
        attr_name: &str,
    ) -> hdf5::Result<()> {
        self.file.group(group_name)?.delete_attr(attr_name)
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
