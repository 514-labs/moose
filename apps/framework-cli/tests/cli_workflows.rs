use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_workflow_init_basic() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("moose-cli").unwrap();

    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("workflow")
        .arg("init")
        .arg("daily-etl")
        .assert();

    // Check for success and workflow directory creation
    assert.success();

    let workflow_dir = temp_dir.path().join("scripts").join("daily-etl");
    assert!(
        workflow_dir.exists(),
        "Workflow directory should be created"
    );
    assert!(workflow_dir.is_dir(), "Workflow path should be a directory");

    // Check config.toml exists and has correct content
    let config_path = workflow_dir.join("config.toml");
    assert!(config_path.exists(), "config.toml should be created");

    let config_content = std::fs::read_to_string(config_path).unwrap();
    assert!(
        config_content.contains("name = 'daily-etl'"),
        "Config should contain workflow name"
    );
}

#[test]
fn test_workflow_init_with_steps() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("moose-cli").unwrap();

    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("workflow")
        .arg("init")
        .arg("daily-etl")
        .arg("--steps=extract,transform,load")
        .assert();

    // Check for success and workflow directory creation
    assert.success();

    let workflow_dir = temp_dir.path().join("scripts").join("daily-etl");
    assert!(
        workflow_dir.exists(),
        "Workflow directory should be created"
    );
    assert!(workflow_dir.is_dir(), "Workflow path should be a directory");

    // Check config.toml exists
    let config_path = workflow_dir.join("config.toml");
    assert!(config_path.exists(), "config.toml should be created");

    // Check step files are created
    let step_files = ["1.extract.py", "2.transform.py", "3.load.py"];

    for step_file in step_files.iter() {
        let file_path = workflow_dir.join(step_file);
        assert!(file_path.exists(), "Step file {} should exist", step_file);

        // Check file content includes basic template
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(
            content.contains("@task"),
            "Step file {} should contain @task decorator",
            step_file
        );
    }

    // Check config.toml exists and has correct content
    let config_path = workflow_dir.join("config.toml");
    assert!(config_path.exists(), "config.toml should be created");

    let config_content = std::fs::read_to_string(config_path).unwrap();

    println!("Config");
    println!("{}", config_content);
    assert!(
        config_content.contains("name = 'daily-etl'"),
        "Config should contain workflow name"
    );
    assert!(
        config_content.contains("steps ="),
        "Config should contain steps section"
    );
    assert!(
        config_content.contains("'extract'"),
        "Config should contain extract"
    );
    assert!(
        config_content.contains("'transform'"),
        "Config should contain transform"
    );
    assert!(
        config_content.contains("'load'"),
        "Config should contain load"
    );
}

#[test]
fn test_workflow_run() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("moose-cli").unwrap();

    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("workflow")
        .arg("run")
        .arg("daily-etl")
        .arg("--input")
        .arg("test-data.csv")
        .assert();

    assert
        .failure()
        .stderr(predicate::str::contains("Not implemented"));

    // Once implemented, should check for:
    // 1. Workflow execution started
    // 2. Input file properly passed
    // 3. Execution status reported
}

#[test]
fn test_workflow_resume() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("moose-cli").unwrap();

    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("workflow")
        .arg("resume")
        .arg("daily-etl")
        .arg("--from")
        .arg("transform")
        .assert();

    assert
        .failure()
        .stderr(predicate::str::contains("Not implemented"));

    // Once implemented, should check for:
    // 1. Workflow resumed from specified step
    // 2. Previous steps skipped
    // 3. Execution status reported
}

#[test]
fn test_workflow_init_with_multiple_step_flags() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("moose-cli").unwrap();

    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("workflow")
        .arg("init")
        .arg("daily-etl")
        .arg("--step")
        .arg("extract")
        .arg("--step")
        .arg("transform")
        .arg("--step")
        .arg("load")
        .assert();

    assert
        .failure()
        .stderr(predicate::str::contains("Not implemented"));

    // Once implemented, should check for:
    // 1. workflows/daily-etl directory created
    // 2. 1.extract.py, 2.transform.py, 3.load.py created
    // 3. config.toml created with proper structure
}
