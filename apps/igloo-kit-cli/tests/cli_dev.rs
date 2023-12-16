use assert_cmd::prelude::*;
use std::fs;
use std::process::Command;

#[test]
fn should_not_run_if_coming_soon_wall_is_blocking() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("igloo-cli")?;

    cmd.env("IGLOO-FEATURES-COMING_SOON_WALL", "true")
        .arg("dev")
        .current_dir(temp.path());

    cmd.assert().success();

    Ok(())
}

#[test]
fn should_properly_get_data_in_storage() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();
    let dir: &str = temp.path().to_str().unwrap();

    // Setup the project with the cli
    let mut init_cmd = Command::cargo_bin("igloo-cli")?;

    init_cmd
        .env("IGLOO-FEATURES-COMING_SOON_WALL", "false")
        .arg("init")
        .arg("test-app")
        .arg("ts")
        .arg(dir);

    init_cmd.assert().success();

    // Run the dev command
    let mut cmd = Command::cargo_bin("igloo-cli")?;

    let res = cmd
        .env("IGLOO-FEATURES-COMING_SOON_WALL", "false")
        .arg("dev")
        .current_dir(&temp)
        .spawn()?;

    fs::copy(
        "tests/psl/simple.prisma",
        format!("{}/app/dataframes/users.prisma", &dir),
    )?;

    Ok(())
}
