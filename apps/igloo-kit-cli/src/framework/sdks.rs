use std::{path::PathBuf, io::Write};

use crate::project::Project;

pub struct CodeFile {
    pub name: String,
    pub code: String,
}

pub fn generate_sdk_dir(project_dir: PathBuf) {
    let sdk_dir = project_dir.join("ts_sdk");
    std::fs::create_dir_all(sdk_dir.clone()).expect("Failed to create sdk directory");
}

pub fn generate_ts_sdk(project: &Project, ts_files: Vec<CodeFile>) -> Result<(), std::io::Error> {
    let package_json = todo!("Generate package.json");
    let index = todo!("Generate index.ts");
    let ts_config = todo!("Generate tsconfig.json");

    for code_file in ts_files {
        let path = project.location.join("ts_sdk").join(code_file.name);
        let mut file = std::fs::File::create(path)?;
        file.write_all(code_file.code.as_bytes())?;
    };

}