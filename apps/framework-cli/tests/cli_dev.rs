use assert_cmd::prelude::*;
use assert_fs::TempDir;
use lazy_static::lazy_static;
use std::fs;
// use std::io::{BufRead, BufReader};
use std::process::Child;
use std::process::{Command, Stdio};
use std::time;
use std::time::Duration;

struct CargoDev {
    _dir: TempDir,
    dev: Child,
}

lazy_static! {
    static ref ACCEPTABLE_PERFORMANCE: Duration = time::Duration::from_millis(10000);
}

fn setup_dev() -> Result<CargoDev, anyhow::Error> {
    // This is to set up a new sandbox directory / project to run the test
    // inside of.
    let temp = assert_fs::TempDir::new()?;
    fs::remove_dir(&temp)?;
    let dir = temp.path().to_str().unwrap();

    // Setup the project with the cli
    let mut init_cmd = Command::cargo_bin("moose-cli")?;

    init_cmd
        .arg("init")
        .arg("test-app")
        .arg("ts")
        .arg("--location")
        .arg(dir);

    init_cmd.assert().success();

    let mut cmd = Command::cargo_bin("moose-cli")?;

    let dev_process = cmd
        .arg("dev")
        .stdout(Stdio::piped())
        .current_dir(&temp)
        .spawn()?;

    // Now we wait for the server to be up before moving forward with
    // the rest of the test
    // TODO this never completes - we should figure out why

    // let reader = BufReader::new(dev_process.stdout.take().expect("Failed to open stdout"));
    // for line in reader.lines() {
    //     let line = line.expect("Failed to read line");
    //     // TODO improve to extract the port to make it more stable to used ports
    //     if line.contains("server on port:") {
    //         break;
    //     }
    // }

    Ok(CargoDev {
        _dir: temp,
        dev: dev_process,
    })
}

fn teardown_dev(mut dev_state: CargoDev) {
    dev_state.dev.kill().unwrap();
}

#[test]
fn should_properly_get_data_in_storage() -> Result<(), anyhow::Error> {
    let dev = setup_dev()?;

    // Add test steps here

    teardown_dev(dev);

    Ok(())
}
