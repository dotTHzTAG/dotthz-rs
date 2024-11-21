# Interface with dotThz files using rust

This crate provides an easy way to interface with [dotThz](https://github.com/dotTHzTAG) files in rust.

Load it in your `cargo.toml`

```toml
[dependencies]
dotthz-rs = "0.2.0"
```

and then use like specified in the following example:

```rust
use std::path::PathBuf;
use dotthz::{DotthzFile};

fn main() {
    // Load data from the original file
    let file_path = PathBuf::new("test_files/2_VariableTemperature.thz");
    let file = DotthzFile::create(&file_path);
    
    // do stuff with the file
    // ...

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
    
    // data is now already saved to the file.

}
```

Use the `hdf5-sys-static` feature to compile hdf5 and statically link it. This requires `cmake` to be installed.
Use the `serde` feature to derive `Serialize` and `Deserialize` for `DotthzMetaData`.