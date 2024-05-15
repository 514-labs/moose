use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use flate2::read::GzDecoder;
use futures::StreamExt;
use tar::Archive;

use super::{RoutineFailure, RoutineSuccess};
use crate::cli::display::{Message, MessageType};
use crate::cli::settings::user_directory;

const TEMPLATE_REGISTRY_URL: &str = "https://templates.514.dev";
const DOWLOAD_DIR: &str = "templates";

// TODO - no need to download every time, once cached once, use the cached version
fn templates_download_dir() -> PathBuf {
    let mut path = user_directory();
    path.push(DOWLOAD_DIR);
    path
}

fn template_file_archive(template_name: &str, template_version: &str) -> PathBuf {
    let mut path = templates_download_dir();
    path.push(template_version);
    path.push(format!("{}.tgz", template_name));
    path
}

async fn download(template_name: &str, template_version: &str) -> anyhow::Result<()> {
    let res = reqwest::get(format!(
        "{}/{}/{}.tgz",
        TEMPLATE_REGISTRY_URL, template_version, template_name
    ))
    .await?;

    if res.status() == 404 {
        anyhow::bail!("Template not found")
    }

    res.error_for_status_ref()?;

    let mut dest = templates_download_dir();
    dest.push(template_version);

    // Create the directory if it doesn't exist
    if !dest.exists() {
        std::fs::create_dir_all(&dest)?;
    }

    dest.push(format!("{}.tgz", template_name));

    let mut stream = res.bytes_stream();
    let mut out = File::create(dest)?;
    while let Some(item) = stream.next().await {
        let chunk = item?;
        out.write_all(&chunk)?;
    }

    Ok(())
}

fn unpack(template_name: &str, template_version: &str, target_dir: &PathBuf) -> anyhow::Result<()> {
    let template_archive = template_file_archive(template_name, template_version);
    let tar_gz = File::open(template_archive)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(target_dir)?;

    Ok(())
}

async fn download_and_unpack(
    template_name: &str,
    template_version: &str,
    target_dir: &Path,
) -> anyhow::Result<()> {
    let canonnical_path = target_dir.canonicalize()?;

    download(template_name, template_version).await?;
    unpack(template_name, template_version, &canonnical_path)
}

pub async fn generate_template(
    template_name: &str,
    template_version: &str,
    target_dir: &Path,
) -> Result<RoutineSuccess, RoutineFailure> {
    // In dev we don't have a version, so we use the latest
    let version = if template_version == "0.0.1" {
        "latest".to_string()
    } else {
        template_version.to_string()
    };

    match download_and_unpack(template_name, &version, target_dir).await {
        Ok(()) => {
            show_message!(
                MessageType::Success,
                Message {
                    action: "Created".to_string(),
                    details: "template".to_string(),
                }
            );
            Ok(RoutineSuccess::info(Message::new(
                "Next steps".to_string(),
                format!(
                    "Run these commands to start Moose: cd {}/moose && npx @514labs/moose-cli@latest dev",
                    target_dir.to_string_lossy()
                ),
            )))
        }
        Err(e) => Err(RoutineFailure::error(Message {
            action: "Template".to_string(),
            details: format!("Failed to generate template: {:?}", e),
        })),
    }
}
