use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{self, Write};

/// Write bytes to a file in a specified directory.
///
/// # Arguments
///
/// * `bytes` - The bytes to be written to the file.
/// * `file_name` - The name of the file to be created or overwritten.
/// * `out_dir` - The directory in which the file will be created, if it doesn't exist.
///
/// # Errors
///
/// Returns an `io::Error` if there was any error creating the directory, creating the file,
/// or writing the bytes to the file.
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