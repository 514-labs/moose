mod cli;
pub mod framework;
pub mod infrastructure;
pub mod metrics;
pub mod metrics_inserter;
pub mod project;
pub mod utilities;

pub mod proto;

// This is not Aysnc because we need to have the instrumentrumentation
// before Tokio takes over the main thread.
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { cli::cli_run().await });
}
