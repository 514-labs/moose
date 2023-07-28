use std::{path::PathBuf, io::{Error, ErrorKind}};



fn create_igloo_directory() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()?;
    let igloo_dir = current_dir.join(".igloo");

    std::fs::create_dir_all(igloo_dir.clone())?;

    Ok(igloo_dir)

}

fn check_for_igloo_directory() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()?;
    let igloo_dir = current_dir.join(".igloo");

    if igloo_dir.exists() {
        Ok(igloo_dir)
    } else {
        Err(Error::new(ErrorKind::NotFound, "Igloo directory not found"))
    }
}

fn create_project_directories() {
    
}


pub fn initialize_project() -> Result<PathBuf, Error> {
    match check_for_igloo_directory() {
        Ok(_) => Err(Error::new(ErrorKind::AlreadyExists, "Igloo directory already exists")),
        Err(_) => {
            println!("Didn't find existing igloo directory, creating one now");
            create_igloo_directory()
        }
    }
}