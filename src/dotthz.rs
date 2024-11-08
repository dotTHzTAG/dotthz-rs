use hdf5::types::VarLenUnicode;
use hdf5::File;
use ndarray::Array2;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use indexmap::IndexMap;

/// A structure representing a .thz file according to the dotThz standard
#[derive(Default)]
pub struct DotthzFile {
    /// A map of group names to measurement data.
    pub groups: IndexMap<String, DotthzMeasurement>,
}


/// Metadata associated with a dotThz measurement.
#[derive(Default, Debug)]
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

/// A structure representing a dotThz measurement containing datasets and metadata. (HDF5 group)
#[derive(Default)]
pub struct DotthzMeasurement {
    /// A map of dataset names to data arrays.
    pub datasets: IndexMap<String, Array2<f32>>,

    /// Metadata associated with the measurement.
    pub meta_data: DotthzMetaData,
}

impl DotthzFile {
    /// Creates a new DotthzFile with the provided data and metadata.
    pub fn new(data: Array2<f32>, meta_data: DotthzMetaData) -> Self {
        let mut groups = IndexMap::new();
        let mut datasets = IndexMap::new();
        datasets.insert("ds1".to_string(), data);
        groups.insert("Measurement 1".to_string(), DotthzMeasurement {
            datasets,
            meta_data,
        });
        DotthzFile { groups }
    }

    /// Loads a DotthzFile from the specified path.
    pub fn load(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        // Open the HDF5 file for reading
        let file = File::open(path.clone())?;

        // Retrieve all groups
        let mut groups = IndexMap::new();
        for (group, group_name) in file.groups()?.iter().zip(file.member_names()?) {
            let mut measurement = DotthzMeasurement {
                datasets: IndexMap::new(),
                meta_data: DotthzMetaData::default(),
            };

            if let Ok(ds_description) = group
                .attr("dsDescription")
                .and_then(|a| a.read_raw::<VarLenUnicode>())
            {

                // Convert ds_description to a vector of strings, splitting any single entry by ", "
                let descriptions: Vec<String> = if ds_description.len() == 1 {
                    // If there's only one entry, split it by ", "
                    ds_description[0].split(", ").map(|s| s.to_string()).collect()
                } else {
                    // Otherwise, assume it's already in the correct format
                    ds_description.iter().map(|s| s.to_string()).collect()
                };

                for (i, description) in descriptions.iter().enumerate() {
                    // Read datasets and populate DataContainer fields, skipping any that are missing
                    if let Ok(ds) = group.dataset(format!("ds{}", i + 1).as_str()).and_then(|d| d.read_2d()) {
                        measurement.datasets.insert(description.to_string(), ds);
                    }
                }
            }

            // Read metadata attributes, skipping any that are missing
            if let Ok(description) = group
                .attr("description")
                .and_then(|a| a.read_raw::<VarLenUnicode>())
            {
                if let Some(d) = description.first() {
                    measurement.meta_data.description = d.to_string();
                }
            }

            if let Ok(date) = group
                .attr("date")
                .and_then(|a| a.read_raw::<VarLenUnicode>())
            {
                if let Some(d) = date.first() {
                    measurement.meta_data.date = d.to_string();
                }
            }

            if let Ok(instrument) = group
                .attr("instrument")
                .and_then(|a| a.read_raw::<VarLenUnicode>())
            {
                if let Some(d) = instrument.first() {
                    measurement.meta_data.instrument = d.to_string();
                }
            }

            if let Ok(md_description) = group
                .attr("mdDescription")
                .and_then(|a| a.read_raw::<VarLenUnicode>())
            {
                // Convert ds_description to a vector of strings, splitting any single entry by ", "
                let descriptions: Vec<String> = if md_description.len() == 1 {
                    // If there's only one entry, split it by ", "
                    md_description[0].split(", ").map(|s| s.to_string()).collect()
                } else {
                    // Otherwise, assume it's already in the correct format
                    md_description.iter().map(|s| s.to_string()).collect()
                };

                for (i, description) in descriptions.iter().enumerate() {
                    // now read the mds
                    if let Ok(md) = group.attr(format!("md{}", i + 1).as_str()).and_then(|a| a.read_raw::<f32>()) {
                        if let Some(meta_data) = md.first() {
                            measurement.meta_data.md.insert(description.to_string(), format!("{}", meta_data));
                        }
                    }
                    if let Ok(md) = group.attr(format!("md{}", i + 1).as_str()).and_then(|a| a.read_raw::<VarLenUnicode>()) {
                        if let Some(meta_data) = md.first() {
                            measurement.meta_data.md.insert(description.to_string(), format!("{}", meta_data));
                        }
                    }
                }
            }

            if let Ok(mode) = group
                .attr("mode")
                .and_then(|a| a.read_raw::<VarLenUnicode>())
            {
                if let Some(d) = mode.first() {
                    measurement.meta_data.mode = d.to_string();
                }
            }

            if let Ok(version) = group
                .attr("thzVer")
                .and_then(|a| a.read_raw::<VarLenUnicode>())
            {
                if let Some(d) = version.first() {
                    measurement.meta_data.version = d.to_string();
                }
            }

            if let Ok(time) = group
                .attr("time")
                .and_then(|a| a.read_raw::<VarLenUnicode>())
            {
                if let Some(d) = time.first() {
                    measurement.meta_data.time = d.to_string();
                }
            }

            if let Ok(user_info) = group
                .attr("user")
                .and_then(|a| a.read_raw::<VarLenUnicode>())
            {
                if let Some(d) = user_info.first() {
                    let user_info_str = d.to_string();
                    let user_parts: Vec<&str> = user_info_str.split('/').collect();

                    // Check each part individually to handle cases where fewer than 4 parts are available
                    if let Some(part) = user_parts.get(0) {
                        measurement.meta_data.orcid = part.trim().into();
                    }
                    if let Some(part) = user_parts.get(1) {
                        measurement.meta_data.user = part.trim().into();
                    }
                    if let Some(part) = user_parts.get(2) {
                        measurement.meta_data.email = part.trim().into();
                    }
                    if let Some(part) = user_parts.get(3) {
                        measurement.meta_data.institution = part.trim().into();
                    }
                }
            }
            groups.insert(group_name, measurement);
        }
        Ok(DotthzFile {
            groups,
        })
    }
    
    /// Saves the DotthzFile to the specified path.
    pub fn save_file(&self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let wtr = File::create(&path)?; // open for writing

        for (group_name, measurement) in self.groups.iter() {
            let group = wtr.create_group(&group_name).unwrap();

            // write description of datasets as attribute
            // Join the dataset keys into a single comma-separated string
            let data = measurement.datasets
                .keys()
                .map(|k| k.as_str())
                .collect::<Vec<&str>>()
                .join(", ");

            // Create a single VarLenUnicode instance from the joined string
            let varlen_data = VarLenUnicode::from_str(&data).unwrap();

            // Define the attribute with a shape of 1 (single entry) and write the joined data
            let attr = group
                .new_attr::<VarLenUnicode>()
                .shape(1)
                .create("dsDescription")?;

            // Write the single VarLenUnicode entry as the attribute data
            attr.write(&[varlen_data])?;

            // write all datasets
            for (i, (_name, dataset)) in measurement.datasets.iter().enumerate() {
                let ds = group
                    .new_dataset::<f32>()
                    .shape(dataset.shape())
                    .create(format!("ds{}", i + 1).as_str())?;
                ds.write_raw(&dataset.as_slice().unwrap())?;
            }

            let entry = VarLenUnicode::from_str(&measurement.meta_data.description).unwrap();
            let attr = group.new_attr::<VarLenUnicode>().create("description")?;
            attr.write_scalar(&entry)?;

            let entry = VarLenUnicode::from_str(&measurement.meta_data.date).unwrap();
            let attr = group.new_attr::<VarLenUnicode>().create("date")?;
            attr.write_scalar(&entry)?;

            let entry = VarLenUnicode::from_str(&measurement.meta_data.instrument).unwrap();
            let attr = group.new_attr::<VarLenUnicode>().create("instrument")?;
            attr.write_scalar(&entry)?;

            // write description of md as attribute
            // Join the dataset keys into a single comma-separated string
            let data = measurement.meta_data.md
                .keys()
                .map(|k| k.as_str())
                .collect::<Vec<&str>>()
                .join(", ");

            // Create a single VarLenUnicode instance from the joined string
            let varlen_data = VarLenUnicode::from_str(&data).unwrap();

            // Define the attribute with a shape of 1 (single entry) and write the joined data
            let attr = group
                .new_attr::<VarLenUnicode>()
                .shape(1)
                .create("mdDescription")?;

            // Write the single VarLenUnicode entry as the attribute data
            attr.write(&[varlen_data])?;


            // write all mds
            for (i, (_name, md)) in measurement.meta_data.md.iter().enumerate() {
                if let Ok(number) = f32::from_str(md) {
                    let attr = group.new_attr::<f32>().create(format!("md{}", i + 1).as_str())?;
                    attr.write_scalar(&number)?; // thickness in mm
                } else {
                    let entry = VarLenUnicode::from_str(md).unwrap();
                    let attr = group.new_attr::<VarLenUnicode>().create(format!("md{}", i + 1).as_str())?;
                    attr.write_scalar(&entry)?;
                }
            }

            let entry = VarLenUnicode::from_str(&measurement.meta_data.mode).unwrap();
            let attr = group.new_attr::<VarLenUnicode>().create("mode")?;
            attr.write_scalar(&entry)?;

            let entry = VarLenUnicode::from_str(&measurement.meta_data.version).unwrap();
            let attr = group.new_attr::<VarLenUnicode>().create("thzVer")?;
            attr.write_scalar(&entry)?;

            let entry = VarLenUnicode::from_str(&measurement.meta_data.time).unwrap();
            let attr = group.new_attr::<VarLenUnicode>().create("time")?;
            attr.write_scalar(&entry)?;

            let entry = VarLenUnicode::from_str(
                format!(
                    "{}/{}/{}/{}",
                    measurement.meta_data.orcid, measurement.meta_data.user, measurement.meta_data.email, measurement.meta_data.institution
                )
                    .as_str(),
            )
                .unwrap();

            let attr = group.new_attr::<VarLenUnicode>().create("user")?;
            attr.write_scalar(&entry)?;
        }

        Ok(())
    }
}