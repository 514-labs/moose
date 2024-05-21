use std::process::Stdio;
use tokio::process::{Child, Command};

pub fn run(script: &str, args: &[&str]) -> Result<Child, std::io::Error> {
    let mut base_process = Command::new("npx")
        .arg("--yes")
        .arg("ts-node")
        .arg("--skipProject")
        .arg("-e")
        .arg(script)
        .arg("--");

    for arg in args {
        base_process.arg(arg);
    }

    base_process
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
