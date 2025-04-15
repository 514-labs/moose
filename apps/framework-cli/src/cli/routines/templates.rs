use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use flate2::read::GzDecoder;
use futures::StreamExt;
use tar::Archive;
use toml::Value;

use super::RoutineFailure;
use super::RoutineSuccess;
use crate::cli::display::{Message, MessageType};
use crate::cli::settings::user_directory;

const TEMPLATE_REGISTRY_URL: &str = "https://templates.514.dev";
const DOWNLOAD_DIR: &str = "templates";
const LOCAL_TEMPLATE_DIR: &str = "template-packages";

// Add a new struct to represent template config
#[derive(Debug)]
pub struct TemplateConfig {
    pub language: String,
    pub description: String,
    pub post_install_print: String,
}

impl TemplateConfig {
    fn from_toml(value: &Value) -> Option<Self> {
        Some(TemplateConfig {
            language: value.get("language")?.as_str()?.to_string(),
            description: value.get("description")?.as_str()?.to_string(),
            post_install_print: value.get("post_install_print")?.as_str()?.to_string(),
        })
    }
}

// TODO - no need to download every time, once cached once, use the cached version
fn templates_download_dir() -> PathBuf {
    let mut path = user_directory();
    path.push(DOWNLOAD_DIR);
    path
}

fn template_file_archive(template_name: &str, template_version: &str) -> PathBuf {
    let mut path = templates_download_dir();
    path.push(template_version);
    path.push(format!("{}.tgz", template_name));
    path
}

async fn download_from_local(template_name: &str) -> anyhow::Result<()> {
    let bin_dir = std::env::current_exe()?;
    let local_template_path = bin_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(LOCAL_TEMPLATE_DIR)
        .join(format!("{}.tgz", template_name));

    if !local_template_path.exists() {
        anyhow::bail!("Local template not found. Did you run scripts/package-templates.js?")
    }

    let mut dest: PathBuf = templates_download_dir();
    dest.push("0.0.1"); // In local mode, we always use "latest"

    // Create the directory if it doesn't exist
    if !dest.exists() {
        std::fs::create_dir_all(&dest)?;
    }

    dest.push(format!("{}.tgz", template_name));
    std::fs::copy(local_template_path, dest)?;

    Ok(())
}

async fn download(template_name: &str, template_version: &str) -> anyhow::Result<()> {
    // If we're in test mode (0.0.1), use local templates
    if template_version == "0.0.1" {
        return download_from_local(template_name).await;
    }

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

    // Filter out macOS metadata files during extraction
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        // Skip macOS metadata files
        if path.to_string_lossy().contains("/._") || path.to_string_lossy().starts_with("._") {
            continue;
        }

        entry.unpack_in(target_dir)?;
    }

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
) -> Result<(), RoutineFailure> {
    match download_and_unpack(template_name, template_version, target_dir).await {
        Ok(()) => {
            show_message!(
                MessageType::Success,
                Message {
                    action: "Created".to_string(),
                    details: "template".to_string(),
                }
            );
            Ok(())
        }
        Err(e) => Err(RoutineFailure::error(Message {
            action: "Template".to_string(),
            details: format!("Failed to generate template: {:?}", e),
        })),
    }
}

pub async fn get_template_manifest(template_version: &str) -> anyhow::Result<Value> {
    // If we're in test mode (0.0.1), use local manifest
    if template_version == "0.0.1" {
        let bin_dir = std::env::current_exe()?;
        let manifest_path = bin_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(LOCAL_TEMPLATE_DIR)
            .join("manifest.toml");

        if !manifest_path.exists() {
            anyhow::bail!(
                "Local manifest not found. Did you run scripts/package-templates.js? {}",
                manifest_path.display()
            )
        }
        let content = std::fs::read_to_string(manifest_path)?;
        let manifest: Value = toml::from_str(&content)?;
        Ok(manifest)
    } else {
        let res = reqwest::get(format!(
            "{}/{}/manifest.toml",
            TEMPLATE_REGISTRY_URL, template_version
        ))
        .await?
        .error_for_status()?;

        let content = res.text().await?;
        let manifest: Value = toml::from_str(&content)?;
        Ok(manifest)
    }
}

pub async fn get_template_config(
    template_name: &str,
    template_version: &str,
) -> Result<TemplateConfig, RoutineFailure> {
    let manifest = get_template_manifest(template_version).await.map_err(|e| {
        RoutineFailure::error(Message {
            action: "Template".to_string(),
            details: format!("Failed to load template manifest: {:?}", e),
        })
    })?;

    let templates = manifest.get("templates").ok_or_else(|| {
        RoutineFailure::error(Message {
            action: "Template".to_string(),
            details: "Invalid manifest: missing templates section".to_string(),
        })
    })?;

    // If template not found, create helpful error message with available templates
    if templates.get(template_name).is_none() {
        let available_templates: Vec<String> = templates
            .as_table()
            .map(|table| {
                table
                    .iter()
                    .filter_map(|(name, config)| {
                        TemplateConfig::from_toml(config).map(|config| {
                            format!(
                                "  - {} ({}) - {}",
                                name, config.language, config.description
                            )
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        return Err(RoutineFailure::error(Message {
            action: "Template".to_string(),
            details: format!(
                "Template '{}' not found. Available templates:\n{}",
                template_name,
                available_templates.join("\n")
            ),
        }));
    }

    let template_config = TemplateConfig::from_toml(templates.get(template_name).unwrap())
        .ok_or_else(|| {
            RoutineFailure::error(Message {
                action: "Template".to_string(),
                details: format!("Invalid configuration for template '{}'", template_name),
            })
        })?;

    Ok(template_config)
}

pub async fn list_available_templates(
    template_version: &str,
) -> Result<RoutineSuccess, RoutineFailure> {
    let manifest = get_template_manifest(template_version).await.map_err(|e| {
        RoutineFailure::error(Message {
            action: "Templates".to_string(),
            details: format!("Failed to load template manifest: {:?}", e),
        })
    })?;

    let templates = manifest.get("templates").ok_or_else(|| {
        RoutineFailure::error(Message {
            action: "Templates".to_string(),
            details: "Invalid manifest: missing templates section".to_string(),
        })
    })?;

    let available_templates: Vec<String> = templates
        .as_table()
        .map(|table| {
            table
                .iter()
                .filter_map(|(name, config)| {
                    TemplateConfig::from_toml(config).map(|config| {
                        format!(
                            "  - {} ({}) - {}",
                            name, config.language, config.description
                        )
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let output = format!(
        "Available templates for version {}:
{}",
        template_version,
        available_templates.join(
            "
"
        )
    );

    Ok(RoutineSuccess::success(Message::new(
        "Templates".to_string(),
        output,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::Once;

    static SETUP: Once = Once::new();

    // Helper function to set up the test environment by copying necessary files
    fn setup_test_environment() -> anyhow::Result<()> {
        let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
        let workspace_root = crate_dir.parent().unwrap().parent().unwrap();

        let source_dir = workspace_root.join("template-packages");
        let target_dir = workspace_root.join("target/template-packages");

        // Run the package-templates.js script to generate the templates
        println!("Running scripts/package-templates.js to generate templates...");
        let script_path = workspace_root.join("scripts/package-templates.js");

        let status = Command::new("node")
            .current_dir(workspace_root)
            .arg(script_path)
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to run scripts/package-templates.js");
        }

        if !source_dir.exists() {
            anyhow::bail!(
                "Source template package directory not found even after running scripts/package-templates.js: {:?}",
                source_dir
            );
        }

        // Ensure the target directory exists
        fs::create_dir_all(&target_dir)?;

        // Files to copy (add more if other tests need different template .tgz files)
        let files_to_copy = ["manifest.toml", "default.tgz", "python.tgz"];

        for file_name in files_to_copy {
            let source_file = source_dir.join(file_name);
            let target_file = target_dir.join(file_name);

            if source_file.exists() {
                fs::copy(&source_file, &target_file)?;
            } else {
                // Optionally warn or error if a source file is missing
                eprintln!(
                    "Warning: Source file {} not found, skipping copy.",
                    source_file.display()
                );
            }
        }

        Ok(())
    }

    fn ensure_test_environment() {
        SETUP.call_once(|| {
            setup_test_environment().expect("Failed to set up test environment");
        });
    }

    #[tokio::test]
    async fn test_list_available_templates_local() {
        ensure_test_environment();
        // Use version "0.0.1" to test against the local manifest
        let result = list_available_templates("0.0.1").await;

        assert!(
            result.is_ok(),
            "Failed to list available templates: {:?}",
            result.err()
        );
        let success_message = result.unwrap().message.details;

        // Basic check to see if the output contains expected template info structure
        assert!(success_message.contains("Available templates for version 0.0.1"));
        // Check for specific templates expected in the local manifest
        assert!(success_message.contains("- typescript (typescript)"));
        assert!(success_message.contains("- python (python)"));
    }

    #[tokio::test]
    async fn test_get_template_config_success() {
        ensure_test_environment();
        // Test getting a valid template config ("default") using the local manifest
        let result = get_template_config("typescript", "0.0.1").await;
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.language, "typescript");
        assert_eq!(config.description, "default ts project");
        // Add more assertions if needed for post_install_print etc.
    }

    #[tokio::test]
    async fn test_get_template_config_not_found() {
        ensure_test_environment();
        // Test getting a non-existent template config using the local manifest
        let result = get_template_config("non_existent_template", "0.0.1").await;
        assert!(result.is_err());
        let error_message = result.unwrap_err().message.details;
        assert!(error_message.contains("Template 'non_existent_template' not found"));
        assert!(error_message.contains("Available templates:")); // Check if it lists available templates
    }

    #[tokio::test]
    async fn test_get_template_manifest_local() {
        ensure_test_environment();
        // Test getting the local manifest (version "0.0.1")
        let result = get_template_manifest("0.0.1").await;
        assert!(
            result.is_ok(),
            "Failed to get local manifest: {:?}",
            result.err()
        );
        let manifest = result.unwrap();

        // Check if the manifest has the 'templates' table
        assert!(manifest.get("templates").is_some());
        assert!(manifest["templates"].is_table());

        // Check for the existence of specific template configurations within the 'templates' table
        let templates_table = manifest["templates"].as_table().unwrap();
        assert!(templates_table.contains_key("typescript"));
        assert!(templates_table.contains_key("python"));
    }
}
