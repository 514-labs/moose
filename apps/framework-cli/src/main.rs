mod cli;
pub mod framework;
pub mod infrastructure;
pub mod metrics;
pub mod metrics_inserter;
pub mod project;
pub mod utilities;

pub mod proto;

use tracing_subscriber::prelude::*;

// This is not Aysnc because we need to have sentry instrument
// before Tokio takes over the main thread.
// REF: https://docs.sentry.io/platforms/rust/#asynchronous
fn main() {
    let sentry_dsn_opt = std::env::var("SENTRY_DSN");
    let sentry_trace_sample_rate = std::env::var("SENTRY_TRACE_SAMPLE_RATE")
        .unwrap_or_else(|_| "1.0".to_string())
        .parse::<f32>()
        .expect("Failed to parse SENTRY_TRACE_SAMPLE_RATE");

    // The guard needs to be kept in scope for the entire duration of the program.
    // If the guard is dropped, then the transport that was initialized shuts down
    // and no further events can be sent on it.
    let _guard = if let Ok(sentry_dsn) = sentry_dsn_opt {
        let environment = if cfg!(debug_assertions) {
            "development"
        } else {
            "production"
        };

        Some(sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: sentry_trace_sample_rate,
                environment: Some(environment.into()),
                ..Default::default()
            },
        )))
    } else {
        None
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let subscriber = tracing_subscriber::Registry::default()
                .with(sentry::integrations::tracing::layer());

            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");

            cli::cli_run().await
        });
}
