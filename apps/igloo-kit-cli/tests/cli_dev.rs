use assert_cmd::prelude::*;
use assert_fs::TempDir;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::Child;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{thread, time};

struct CargoDev {
    dir: TempDir,
    dev: Child,
}

lazy_static! {
    static ref ACCEPTABLE_PERFORMANCE: Duration = time::Duration::from_millis(10000);
}

fn setup_dev() -> Result<CargoDev, anyhow::Error> {
    // This is to setup a new sandbox directory / project to run the test
    // inside of.
    let temp = assert_fs::TempDir::new()?;
    let dir = temp.path().to_str().unwrap();

    // Setup the project with the cli
    let mut init_cmd = Command::cargo_bin("igloo-cli")?;

    init_cmd
        .env("IGLOO-FEATURES-COMING_SOON_WALL", "false")
        .arg("init")
        .arg("test-app")
        .arg("ts")
        .arg(dir);

    init_cmd.assert().success();

    let mut cmd = Command::cargo_bin("igloo-cli")?;

    let mut dev_process = cmd
        .env("IGLOO-FEATURES-COMING_SOON_WALL", "false")
        .arg("dev")
        .stdout(Stdio::piped())
        .current_dir(&temp)
        .spawn()?;

    // Now we wait for the server to be up before moving forward with
    // the rest of the test
    let reader = BufReader::new(dev_process.stdout.take().expect("Failed to open stdout"));
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        // TODO improve to extract the port to make it more stable to used ports
        if line.contains("server on port:") {
            break;
        }
    }

    Ok(CargoDev {
        dir: temp,
        dev: dev_process,
    })
}

fn teardown_dev(mut dev_state: CargoDev) {
    dev_state.dev.kill().unwrap();
}

#[test]
fn should_not_run_if_coming_soon_wall_is_blocking() -> Result<(), anyhow::Error> {
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("igloo-cli")?;

    cmd.env("IGLOO-FEATURES-COMING_SOON_WALL", "true")
        .arg("dev")
        .current_dir(temp.path());

    cmd.assert().success();

    Ok(())
}

#[test]
fn should_properly_get_data_in_storage() -> Result<(), anyhow::Error> {
    let dev = setup_dev()?;

    let test_steps = || -> Result<(), anyhow::Error> {
        fs::copy(
            "tests/psl/simple.prisma",
            format!(
                "{}/app/dataframes/users.prisma",
                dev.dir.path().to_str().unwrap()
            ),
        )?;

        // TODO Add the test in which we look for the result of copying the file
        // with the API used to show the console.
        // TODO add sending an event to the server and seeing the data getting ingested
        // by the storage layer

        // thread::sleep(*ACCEPTABLE_PERFORMANCE);

        // let resp = reqwest::blocking::get("http://localhost:4000/console")?
        //     .json::<HashMap<String, String>>()?;

        // println!("{:#?}", resp);

        Ok(())
    };

    let test_res = test_steps();

    teardown_dev(dev);

    test_res
}
