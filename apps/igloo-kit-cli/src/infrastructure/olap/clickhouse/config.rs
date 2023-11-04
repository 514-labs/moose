//! # Clickhouse Config
//! Module to handle the creation of the Clickhouse config files
//!
//! ## Suggested Improvements
//! - we need to understand clickhouse configuration better before we can go deep on it's configuration
//!

use serde::Deserialize;

use crate::infrastructure::PANDA_NETWORK;

#[derive(Deserialize, Debug, Clone)]
pub struct ClickhouseConfig {
    pub db_name: String, // ex. local
    pub user: String,
    pub password: String,
    pub host: String,            // ex. localhost
    pub host_port: i32,          // ex. 18123
    pub postgres_port: i32,      // ex. 9005
    pub kafka_port: i32,         // ex. 9092
    pub cluster_network: String, // ex. panda-house
}

impl Default for ClickhouseConfig {
    fn default() -> Self {
        Self {
            db_name: "local".to_string(),
            user: "panda".to_string(),
            password: "pandapass".to_string(),
            host: "localhost".to_string(),
            host_port: 18123,
            postgres_port: 9005,
            kafka_port: 9092,
            cluster_network: PANDA_NETWORK.to_owned(),
        }
    }
}

// *** The following configurations work. They just didn't solve the problem I needed them to solve. ***
//
// We should continue to explore them
//
// use std::fs::File;
// use std::io::prelude::*;
// use std::path::PathBuf;

// pub fn create_server_config_file(directory: &PathBuf) -> std::io::Result<()> {
//     let mut file = File::create(directory.join("psql.xml"))?;
//     file.write_all(b"<clickhouse><postgresql_port>9005</postgresql_port></clickhouse>")?;

//     Ok(())
// }

// pub fn create_user_config_file(directory: &PathBuf) -> std::io::Result<()> {
//     let mut file = File::create(directory.join("panda.xml"))?;
//     file.write_all(b"
//     <clickhouse>
//         <profiles>
//             <experimental>
//                 <allow_experimental_object_type>0</allow_experimental_object_type>
//             </experimental>
//         </profiles>
//         <users>
//             <panda>
//                 <profile>experimental</profile>
//             </panda>
//         </users>
//     </clickhouse>
//     ")?;

//     Ok(())
// }

// pub fn create_init_script(directory: &PathBuf) -> std::io::Result<()> {
//     let mut file = File::create(directory.join("db.sql"))?;
//     file.write_all(b"
//     SET allow_experimental_object_type = 1;
//     ")?;

//     Ok(())
// }
