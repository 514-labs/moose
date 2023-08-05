use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn create_config_file(directory: &PathBuf) -> std::io::Result<()> {
    let mut file = File::create(directory.join("psql.xml"))?;
    file.write_all(b"<clickhouse><postgresql_port>9005</postgresql_port></clickhouse>")?;
    Ok(())
}