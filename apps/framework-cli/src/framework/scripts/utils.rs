#[derive(Debug, thiserror::Error)]
pub enum TemporalExecutionError {
    #[error("Temportal connection error: {0}")]
    TemporalConnectionError(#[from] tonic::transport::Error),

    #[error("Temportal client error: {0}")]
    TemporalClientError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

/// Parses various schedule formats into a valid Temporal cron expression
///
/// # Arguments
/// * `schedule` - Optional string containing the schedule format
///
/// # Returns
/// A String containing the parsed cron expression or empty string if invalid
///
/// # Formats Supported
/// * Standard cron expressions (e.g., "* * * * *")
/// * Interval notation (e.g., "*/5 * * * *")
/// * Simple duration formats:
///   - "5m" → "*/5 * * * *" (every 5 minutes)
///   - "2h" → "0 */2 * * *" (every 2 hours)
///
/// Falls back to empty string (no schedule) if format is invalid
pub fn parse_schedule(schedule: &str) -> String {
    if schedule.is_empty() {
        return String::new();
    }

    match schedule {
        // Handle interval-based formats
        s if s.contains('/') => s.to_string(),
        // Handle standard cron expressions
        s if s.contains('*') || s.contains(' ') => s.to_string(),
        // Convert simple duration to cron (e.g., "5m" -> "*/5 * * * *")
        s if s.ends_with('m') => {
            let mins = s.trim_end_matches('m');
            format!("*/{} * * * *", mins)
        }
        s if s.ends_with('h') => {
            let hours = s.trim_end_matches('h');
            format!("0 */{} * * *", hours)
        }
        // Default to original string if format is unrecognized
        s => s.to_string(),
    }
}

pub fn parse_timeout_to_seconds(timeout: &str) -> Result<i64, TemporalExecutionError> {
    if timeout.is_empty() {
        return Err(TemporalExecutionError::TimeoutError(
            "Timeout string is empty".to_string(),
        ));
    }

    let (value, unit) = timeout.split_at(timeout.len() - 1);
    let value: u64 = value
        .parse()
        .map_err(|_| TemporalExecutionError::TimeoutError("Invalid number format".to_string()))?;

    let seconds =
        match unit {
            "h" => value * 3600,
            "m" => value * 60,
            "s" => value,
            _ => return Err(TemporalExecutionError::TimeoutError(
                "Invalid time unit. Must be h, m, or s for hours, minutes, or seconds respectively"
                    .to_string(),
            )),
        };

    Ok(seconds as i64)
}

pub fn get_temporal_domain_name(temporal_url: &str) -> &str {
    temporal_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split(':')
        .next()
        .unwrap_or("")
}

pub fn get_temporal_namespace(domain_name: &str) -> String {
    domain_name
        .strip_suffix(".tmprl.cloud")
        .unwrap_or(domain_name)
        .to_string()
}
