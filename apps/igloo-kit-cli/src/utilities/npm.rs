use std::{process::Command, path::PathBuf};


fn get_root() -> Result<PathBuf, std::io::Error> {
    let result = Command::new("npm")
        .arg("root")
        .arg("-g")
        .output()?;

    let stdout = String::from_utf8(result.stdout).expect("Failed to get npm root. Is npm installed?");

    Ok(PathBuf::from(stdout.trim()))
}