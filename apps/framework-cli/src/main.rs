mod cli;
pub mod framework;
pub mod infrastructure;
pub mod project;
pub mod utilities;

// This is not Aysnc because we need to have sentry instrument
// before Tokio takes over the main thread.
// REF: https://docs.sentry.io/platforms/rust/#asynchronous
fn main() {
    let envionment = if cfg!(debug_assertions) {
        "development"
    } else {
        "production"
    };

    if envionment == "production" {
        let _guard = sentry::init(
            ("https://83941f36fe439ebe1ffd4d35b834ed7a@o4505851966128128.ingest.sentry.io/4505851967963136", 
            sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 1.0,
            environment: Some(envionment.into()),
            ..Default::default()
          }));
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(cli::cli_run());
}
