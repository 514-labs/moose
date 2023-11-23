use assert_cmd::prelude::*; // Add methods on commands
use assert_fs::prelude::*;
use predicates::prelude::*; // Used for writing assertions
use std::fs;
use std::process::Command; // Run programs

#[test]
fn cannot_run_igloo_init_without_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("igloo-cli")?;

    cmd.arg("init");
    cmd.assert().failure().stderr(predicate::str::contains(
        "the following required arguments were not provided:",
    ));

    Ok(())
}

#[test]
fn can_run_igloo_init() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();
    let dir = temp.path().to_str().unwrap();
    println!("dir: {}", dir);

    // List the content of dir
    temp.child(".igloo").assert(predicate::path::missing());

    let mut cmd = Command::cargo_bin("igloo-cli")?;

    cmd.arg("init").arg("test-app").arg("ts").arg(dir);

    cmd.assert().success();

    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        println!("{}", entry.path().display());
    }

    // TODO add more specific tests when the layout of the
    // app is more stable
    temp.child(".igloo").assert(predicate::path::exists());
    temp.child("app").assert(predicate::path::exists());

    Ok(())
}
