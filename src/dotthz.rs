use hdf5::types::VarLenUnicode;
use hdf5::File;
use ndarray::Array2;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use indexmap::IndexMap;

#[derive(Default)]
pub struct DotthzFile {
    pub groups: IndexMap<String, DotthzMeasurement>,
}


#[derive(Default)]
pub struct DotthzMetaData {
    pub user: String,
    pub email: String,
    pub orcid: String,
    pub institution: String,
    pub description: String,
    pub md: IndexMap<String, String>,
    pub version: String,
    pub mode: String,
    pub instrument: String,
    pub time: String,
    pub date: String,
}

#[derive(Default)]
pub struct DotthzMeasurement {
    pub datasets: IndexMap<String, Array2<f32>>,
    pub meta_data: DotthzMetaData,
}

impl DotthzFile {
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
                for (i, description) in ds_description.iter().enumerate() {
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
                for (i, description) in md_description.iter().enumerate() {
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
    pub fn save_file(&self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let wtr = File::create(&path)?; // open for writing

        for (group_name, measurement) in self.groups.iter() {
            let group = wtr.create_group(&group_name).unwrap();

            // write description of datasets as attribute
            let data = measurement.datasets.keys().map(|k| VarLenUnicode::from_str(k).unwrap()).collect::<Vec<VarLenUnicode>>();
            let attr = group
                .new_attr::<VarLenUnicode>()
                .shape(measurement.datasets.len())
                .create("dsDescription")?;
            // write the attr data
            attr.write(&data)?;

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
            let data = measurement.meta_data.md.keys().map(|k| VarLenUnicode::from_str(k).unwrap()).collect::<Vec<VarLenUnicode>>();
            let attr = group
                .new_attr::<VarLenUnicode>()
                .shape(measurement.datasets.len())
                .create("mdDescription")?;
            // write the attr data
            attr.write(&data)?;

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