//! System utilities

use std::{fs, io, path::Path, path::PathBuf, process::Command};

pub fn copy_directory(source: &PathBuf, destination: &PathBuf) -> Result<(), std::io::Error> {
    //! Recursively copy a directory from source to destination.
    let mut command = Command::new("cp");
    command.arg("-r");
    command.arg(source);
    command.arg(destination);

    command.output()?;

    Ok(())
}

pub fn read_directory<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    //! Read a directory and print out the names of the directories within it.
    //! Note this function does not use recursion to read subdirectories.
    let entries = fs::read_dir(path)?.collect::<Result<Vec<_>, io::Error>>()?;
    let mut dirs = Vec::new();

    for entry in entries {
        let entry = entry;
        let path = entry.path();
        if path.is_dir() {
            if let Some(file_name) = path.file_name() {
                if let Some(name) = file_name.to_string_lossy().to_string().into() {
                    dirs.push(name);
                }
            }
        }
    }

    Ok(dirs)
}
