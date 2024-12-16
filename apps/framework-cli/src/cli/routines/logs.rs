use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use crate::cli::display::{Message, MessageType};

use super::{RoutineFailure, RoutineSuccess};

pub fn show_logs(log_file_path: String, filter: String) -> Result<RoutineSuccess, RoutineFailure> {
    let child = Command::new("tail")
        .arg(log_file_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to show logs".to_string()),
                err,
            )
        })?;

    let output = child.wait_with_output().map_err(|err| {
        RoutineFailure::new(
            Message::new("Failed".to_string(), "to show logs".to_string()),
            err,
        )
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().filter(|line| line.contains(&filter));
    for line in lines {
        show_message!(
            MessageType::Info,
            Message {
                action: "Log".to_string(),
                details: line.to_string(),
            },
            true
        );
    }

    Ok(RoutineSuccess::success(Message::new(
        "".to_string(),
        "".to_string(),
    )))
}

pub fn follow_logs(
    log_file_path: String,
    filter: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    let child = Command::new("tail")
        .arg("-f")
        .arg(log_file_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to show logs".to_string()),
                err,
            )
        })?;

    if let Some(out) = child.stdout {
        let reader = BufReader::new(out);
        for line_result in reader.lines() {
            let line = match line_result {
                Ok(line) => line,
                Err(err) => {
                    return Err(RoutineFailure::new(
                        Message::new("Failed".to_string(), "to read line from logs".to_string()),
                        err,
                    ))
                }
            };

            if line.contains(&filter) {
                show_message!(
                    MessageType::Info,
                    Message {
                        action: "Log".to_string(),
                        details: line.clone(),
                    },
                    true
                );
            }
        }
    }

    Ok(RoutineSuccess::success(Message::new(
        "".to_string(),
        "".to_string(),
    )))
}
