use std::{fs, path::PathBuf, io::Error};

use super::database;

// Validate that the clickhouse and redpanda volume paths exist
pub fn validate_mount_volumes(igloo_dir: &PathBuf) -> Result<(), String> {
    let panda_house = igloo_dir.join(".panda_house").exists();
    let clickhouse = igloo_dir.join(".clickhouse").exists();

    if panda_house && clickhouse {
        Ok(())
    } else {
        Err(format!("Mount volume status: redpanda: {panda_house}, clickhouse: {clickhouse}"))
    }
}


pub fn delete_red_panda_mount_volume(igloo_dir: &PathBuf) -> Result<(), Error> {
    let mount_dir = igloo_dir.join(".panda_house");
    let output = fs::remove_dir_all(mount_dir.clone());

    match output {
        Ok(_) =>{ 
            println!("Removed mount directory at {}", mount_dir.display());
            Ok(())
        },
        Err(err) => {
            println!("Failed to remove mount directory at {}", mount_dir.display());
            println!("error: {}", err);
            Err(err)
        },
    }
}

pub fn delete_clickhouse_mount_volume(igloo_dir: &PathBuf) -> Result<(), Error> {
    let mount_dir = igloo_dir.join(".clickhouse");
    let output = fs::remove_dir_all(mount_dir.clone());

    match output {
        Ok(_) => {
            println!("Removed mount directory at {}", mount_dir.display());
            Ok(())
        },
        Err(err) => {
            println!("Failed to remove mount directory at {}", mount_dir.display());
            println!("error: {}", err);
            Err(err)
        },
    }
}

pub fn create_red_panda_mount_volume(igloo_dir: &PathBuf) -> Result<PathBuf, Error> {
    let mount_dir = igloo_dir.join(".panda_house");
    
    let output = fs::create_dir_all(mount_dir.clone());

    match output {
        Ok(_) => {
            Ok(mount_dir)
        },
        Err(err) => {
            Err(err)
        },
    }
}

pub fn create_clickhouse_mount_volume(igloo_dir: &PathBuf) -> Result<(), Error> {
    let mount_dir = igloo_dir.join(".clickhouse");

    fs::create_dir_all(mount_dir.clone())?;
    fs::create_dir_all(mount_dir.clone().join("data"))?;
    fs::create_dir_all(mount_dir.clone().join("logs"))?;

    let config_path = mount_dir.clone().join("configs");
    fs::create_dir_all(&config_path)?;
    database::create_config_file(&config_path)?;
    Ok(())
}