mod infrastructure;
mod cli;
pub mod framework;

#[tokio::main]
async fn main() {
    cli::cli_run().await;
}

