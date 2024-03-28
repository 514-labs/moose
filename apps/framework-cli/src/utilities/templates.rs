use super::constants::CLI_VERSION;
use async_recursion::async_recursion;
use octocrab::Octocrab;

const GITHUB_ORG: &str = "514-labs";
const GITHUB_FRAMEWORK_REPO: &str = "moose";
const GITHUB_TEMPLATES_DIR: &str = "templates";

pub async fn download_template_files(template_name: &str, output_dir: &str) -> anyhow::Result<()> {
    let octocrab = Octocrab::builder().build()?;

    let head_reference = if CLI_VERSION != "0.0.1" {
        format!("v{}", CLI_VERSION)
    } else {
        "main".to_string()
    };

    let content = octocrab
        .repos(GITHUB_ORG, GITHUB_FRAMEWORK_REPO)
        .get_content()
        .path(GITHUB_TEMPLATES_DIR)
        .r#ref(&head_reference)
        .send()
        .await?;

    let template = content.items.iter().find(|item| item.name == template_name);

    match template {
        Some(item) => {
            let path = format!("{}/{}", GITHUB_TEMPLATES_DIR, template_name);
            println!("File path: {:?}/{}", path, item.name);
            println!("Download URL is: {:?}", item.download_url);
            println!("content is there {:?}", item.content.is_some());

            download_directory(&octocrab, &path, &head_reference, output_dir).await?;
            Ok(())
        }
        None => Err(anyhow::anyhow!("Template not found")),
    }

    // let template = repo.contents().download("templates/rust-cli.zip").await?;

    // let mut archive = zip::ZipArchive::new(std::io::Cursor::new(template))?;
    // for i in 0..archive.len() {
    //     let mut file = archive.by_index(i)?;
    //     let outpath = std::path::Path::new(output_dir).join(file.sanitized_name());

    //     if file.is_dir() {
    //         std::fs::create_dir_all(&outpath)?;
    //     } else {
    //         if let Some(p) = outpath.parent() {
    //             if !p.exists() {
    //                 std::fs::create_dir_all(&p)?;
    //             }
    //         }
    //         let mut outfile = std::fs::File::create(&outpath)?;
    //         std::io::copy(&mut file, &mut outfile)?;
    //     }
    // }
}

#[async_recursion]
async fn download_directory(
    octocrab: &Octocrab,
    path: &str,
    head: &str,
    output_dir: &str,
) -> anyhow::Result<()> {
    let items = octocrab
        .repos(GITHUB_ORG, GITHUB_FRAMEWORK_REPO)
        .get_content()
        .path(GITHUB_TEMPLATES_DIR)
        .r#ref(head)
        .send()
        .await?;

    for item in items.items {
        if item.r#type == "dir" {
            let new_path = format!("{}/{}", path, item.name);
            let new_output_dir = format!("{}/{}", output_dir, item.name);
            download_directory(octocrab, &new_path, head, &new_output_dir).await?;
        } else {
            println!("File path: {:?}/{}", path, item.name);
            println!("Download URL is: {:?}", item.download_url);
            println!("content is there {:?}", item.content.is_some());
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_download_template_files() {
    download_template_files("mixplank", "/Users/nicolas/test-output")
        .await
        .unwrap();
}
