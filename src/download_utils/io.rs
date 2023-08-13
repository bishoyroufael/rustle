use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{self, Write};

pub fn write_bytes_to_file_in_dir(
    bytes: &bytes::Bytes,
    file_name: &str,
    out_dir: &PathBuf,
) -> Result<(), io::Error> {
    // Create the output directory if it doesn't exist
    fs::create_dir_all(out_dir)?;

    let file_path = out_dir.join(file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(bytes)?;

    Ok(())
}





