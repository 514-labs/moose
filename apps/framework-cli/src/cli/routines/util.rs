use crate::utilities::docker::DockerClient;

pub fn ensure_docker_running(docker_client: &DockerClient) -> anyhow::Result<()> {
    let errors = docker_client.check_status()?;

    if errors.is_empty() {
        Ok(())
    } else if errors
        .iter()
        .any(|s| s.ends_with("Is the docker daemon running?"))
    {
        anyhow::bail!("Failed to run docker commands. Is docker running?")
    } else {
        anyhow::bail!("Failed to run docker commands. {}", errors.join("\n"))
    }
}
