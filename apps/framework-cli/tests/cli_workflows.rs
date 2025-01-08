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

    let workflow_dir = temp_dir
        .path()
        .join("app")
        .join("scripts")
        .join("daily-etl");
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

#[tokio::test]
async fn test_workflow_run() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let app_scripts_dir = temp_dir.path().join("app").join("scripts");
    std::fs::create_dir_all(&app_scripts_dir)?;
    std::env::set_current_dir(&temp_dir)?;

    // First initialize a workflow with steps
    let mut init_cmd = Command::cargo_bin("moose-cli")?;
    init_cmd
        .current_dir(&temp_dir)
        .arg("workflow")
        .arg("init")
        .arg("test-etl")
        .arg("--steps=extract,transform,load")
        .assert()
        .success();

    // Verify workflow directory exists
    let workflow_dir = app_scripts_dir.join("test-etl");
    assert!(workflow_dir.exists(), "Workflow directory should exist");

    // Run the workflow
    let mut run_cmd = Command::cargo_bin("moose-cli")?;
    run_cmd
        .current_dir(&temp_dir)
        .arg("workflow")
        .arg("run")
        .arg("test-etl")
        .assert()
        .success();

    Ok(())
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

    // Check for success and workflow directory creation
    assert.success();

    let workflow_dir = temp_dir.path().join("scripts").join("daily-etl");
    assert!(
        workflow_dir.exists(),
        "Workflow directory should be created"
    );
    assert!(workflow_dir.is_dir(), "Workflow path should be a directory");

    // Check step files are created
    let step_files = ["1.extract.py", "2.transform.py", "3.load.py"];
    for step_file in step_files.iter() {
        let file_path = workflow_dir.join(step_file);
        assert!(file_path.exists(), "Step file {} should exist", step_file);

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
        "Config should contain extract step"
    );
    assert!(
        config_content.contains("'transform'"),
        "Config should contain transform step"
    );
    assert!(
        config_content.contains("'load'"),
        "Config should contain load step"
    );
}
