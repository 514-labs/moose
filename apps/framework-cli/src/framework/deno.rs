static SERVER_FILE: &str = include_str!("server.ts");

pub async fn init_deno_service() -> anyhow::Result<()> {
    // write SERVER_FILE to internal directory
    Ok(())
}
