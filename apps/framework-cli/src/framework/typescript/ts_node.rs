use std::process::Child;
use std::process::Command;
use std::process::Stdio;

pub fn run(script: &str, args: &[&str]) -> Result<Child, std::io::Error> {
    let mut command = Command::new("npx");

    command
        .arg("--yes")
        .arg("ts-node")
        .arg("--skipProject")
        .arg("-e")
        .arg(script)
        .arg("--");

    for arg in args {
        command.arg(arg);
    }

    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
