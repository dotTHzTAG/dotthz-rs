# Interface with dotThz files using rust

This crate provides an easy way to interface with [dotThz](https://github.com/dotTHzTAG) files in rust.

Load it in your `cargo.toml`

```toml
[dependencies]
dotthz-rs = "0.1.1"
```

and then use like specified in the following example:

```rust
use std::path::PathBuf;
use dotthz::{DotthzFile};

fn main() {
    // Load data from the original file
    let file_path = PathBuf::from("test_files/2_VariableTemperature.thz");
    let file = DotthzFile::load(file_path.clone());
    
    // do stuff with the file
    // ...
    
    // save file
    file.save("test_files/output_file.thz");
}
```