use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn create_server_config_file(directory: &PathBuf) -> std::io::Result<()> {
    let mut file = File::create(directory.join("psql.xml"))?;
    file.write_all(b"<clickhouse><postgresql_port>9005</postgresql_port></clickhouse>")?;

    Ok(())
}

pub fn create_user_config_file(directory: &PathBuf) -> std::io::Result<()> {
    let mut file = File::create(directory.join("panda.xml"))?;
    file.write_all(b"
    <clickhouse>
        <profiles>
            <experimental>
                <allow_experimental_object_type>0</allow_experimental_object_type>
            </experimental>
        </profiles>
        <users>
            <panda>
                <profile>experimental</profile>
            </panda>
        </users>
    </clickhouse>
    ")?;

    Ok(())
}


pub fn create_init_script(directory: &PathBuf) -> std::io::Result<()> {
    let mut file = File::create(directory.join("db.sql"))?;
    file.write_all(b"
    SET allow_experimental_object_type = 1;
    ")?;

    Ok(())
}