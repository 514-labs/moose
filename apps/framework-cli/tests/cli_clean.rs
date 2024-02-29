use assert_cmd::prelude::*;
use assert_fs::{prelude::*, TempDir};
use predicates::prelude::*;
use std::{fs, process::Command};

#[test]
fn can_run_cli_clean() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let dir: &str = temp.path().to_str().unwrap();

    temp.child(".moose").assert(predicate::path::missing());

    let mut init_cmd = Command::cargo_bin("moose-cli")?;

    init_cmd
        .env("MOOSE-FEATURES-COMING_SOON_WALL", "false")
        .arg("init")
        .arg("test-app")
        .arg("ts")
        .arg(dir);

    init_cmd.assert().success();

    validate_dotmoose_dir(&temp, true);

    let mut clean_cmd = Command::cargo_bin("moose-cli")?;

    clean_cmd
        .env("MOOSE-FEATURES-COMING_SOON_WALL", "false")
        .arg("clean")
        .current_dir(&temp);

    clean_cmd.assert().success();

    validate_dotmoose_dir(&temp, false);

    Ok(())
}

fn validate_dotmoose_dir(temp: &TempDir, should_exist: bool) {
    println!("should_exist: {}", should_exist);
    let paths = fs::read_dir(temp.child(".moose")).unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }

    let assert_value = if should_exist {
        predicate::path::exists()
    } else {
        predicate::path::missing()
    };
    temp.child(".moose/models/typescript").assert(assert_value);
    temp.child(".moose/.clickhouse").assert(assert_value);
    temp.child(".moose/.panda_house").assert(assert_value);
}
