use crate::cli::display::{show_message_wrapper, Message, MessageType};

/// Helper function to check if a line should be handled as a structured log
pub fn is_structured_log(line: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() == 2 {
        Some((parts[0].trim(), parts[1].trim()))
    } else {
        None
    }
}

/// Helper function to check if a line should be flushed immediately
pub fn should_flush_immediately(line: &str) -> bool {
    line.contains("[CompilerPlugin]")
}

/// Helper function to flush a buffer of lines
pub fn flush_buffer_lines(buffer: &mut Vec<String>, action_name: &str) {
    if !buffer.is_empty() {
        let combined = buffer.join("\n");
        if !combined.trim().is_empty() {
            log::info!("{}", combined);
            show_message_wrapper(
                MessageType::Info,
                Message {
                    action: action_name.to_string(),
                    details: combined,
                },
            );
        }
        buffer.clear();
    }
}

/// Creates a buffered stdout reader with proper service context
///
/// This macro generates the buffering logic inline while preserving the service's logging context.
/// Usage: `buffered_stdout_reader!(stdout, timeout_ms, action_name)`
#[macro_export]
macro_rules! buffered_stdout_reader {
    ($stdout:expr, $timeout_ms:expr, $action_name:expr) => {{
        use $crate::utilities::buffered_stdout_reader::{
            is_structured_log, should_flush_immediately, flush_buffer_lines
        };
        use $crate::cli::display::{show_message_wrapper, Message, MessageType};
        use tokio::io::{AsyncBufReadExt, BufReader};

        let mut stdout_reader = BufReader::new($stdout).lines();

        tokio::spawn(async move {
            let mut buffer = Vec::<String>::new();
            let mut last_activity = tokio::time::Instant::now();

            loop {
                tokio::select! {
                    line_result = stdout_reader.next_line() => {
                        match line_result {
                            Ok(Some(line)) => {
                                last_activity = tokio::time::Instant::now();

                                if let Some((level, message)) = is_structured_log(&line) {
                                    // Flush buffer before structured log
                                    flush_buffer_lines(&mut buffer, $action_name);

                                    // Handle structured log with proper context
                                    match level {
                                        "INFO" => log::info!("{}", message),
                                        "WARN" => log::warn!("{}", message),
                                        "DEBUG" => log::debug!("{}", message),
                                        "ERROR" => {
                                            log::error!("{}", message);
                                            show_message_wrapper(
                                                MessageType::Error,
                                                Message {
                                                    action: $action_name.to_string(),
                                                    details: message.to_string(),
                                                },
                                            );
                                        }
                                        _ => log::info!("{}", message),
                                    }
                                } else if line.trim().is_empty() {
                                    // Skip empty lines
                                } else if should_flush_immediately(&line) {
                                    // Flush buffer and log immediately
                                    flush_buffer_lines(&mut buffer, $action_name);
                                    log::info!("{}", line);
                                } else {
                                    // Buffer the line
                                    buffer.push(line);
                                }
                            }
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }

                    _ = tokio::time::sleep(tokio::time::Duration::from_millis($timeout_ms)) => {
                        if !buffer.is_empty() && last_activity.elapsed().as_millis() >= $timeout_ms as u128 {
                            flush_buffer_lines(&mut buffer, $action_name);
                        }
                    }
                }
            }

            // Final flush
            flush_buffer_lines(&mut buffer, $action_name);
        })
    }};
}
