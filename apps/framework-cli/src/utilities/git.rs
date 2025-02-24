use std::path::Path;
use std::sync::Arc;

use crate::framework::languages::SupportedLanguages;
use git2::{Error, Repository, RepositoryInitOptions, Signature};
use serde::{Deserialize, Serialize};

use crate::project::Project;

use super::constants::{CLI_USER_DIRECTORY, GITIGNORE};

fn default_branch() -> String {
    "main".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitConfig {
    #[serde(default = "default_branch")]
    pub main_branch_name: String,
}

impl Default for GitConfig {
    fn default() -> GitConfig {
        GitConfig {
            main_branch_name: default_branch(),
        }
    }
}

pub fn is_git_repo(dir_path: &Path) -> Result<bool, Error> {
    match Repository::discover(dir_path) {
        Ok(_) => Ok(true),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn create_init_commit(project: Arc<Project>, dir_path: &Path) {
    let mut git_ignore_file = project.project_location.clone();
    git_ignore_file.push(GITIGNORE);

    let mut git_ignore_entries = vec![CLI_USER_DIRECTORY];
    git_ignore_entries.append(&mut match project.language {
        SupportedLanguages::Typescript => {
            vec!["node_modules", "dist", "coverage"]
        }
        SupportedLanguages::Python => vec![
            "__pycache__",
            "*.pyc",
            "*.pyo",
            "*.pyd",
            ".Python",
            "env",
            ".venv",
            "venv",
            "ENV",
            "env.bak",
            ".spyderproject",
            ".ropeproject",
            ".idea",
            "*.ipynb_checkpoints",
            ".pytest_cache",
            ".mypy_cache",
            ".hypothesis",
            ".coverage",
            "cover",
            "*.cover",
            ".DS_Store",
            ".cache",
            "*.so",
            "*.egg",
            "*.egg-info",
            "dist",
            "build",
            "develop-eggs",
            "downloads",
            "eggs",
            "lib",
            "lib64",
            "parts",
            "sdist",
            "var",
            "wheels",
            "*.egg-info/",
            ".installed.cfg",
            "*.egg",
            "MANIFEST",
        ],
    });
    let mut git_ignore = git_ignore_entries.join("\n");
    git_ignore.push_str("\n\n");
    std::fs::write(git_ignore_file, git_ignore).unwrap();

    let mut repo_create_options = RepositoryInitOptions::new();
    repo_create_options.initial_head("main");
    let repo = Repository::init_opts(dir_path, &repo_create_options)
        .expect("Failed to initialize git repo");

    let author =
        Signature::now("Moose CLI", "noreply@fiveonefour.com").expect("Failed to create signature");

    // Now let's create an empty tree for this commit
    let mut index = repo.index().expect("Failed to get repo index");

    index
        .add_all(["."], git2::IndexAddOption::DEFAULT, None)
        .expect("Failed to add path to index");

    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");

    // empty parent because it's the first commit
    repo.commit(Some("HEAD"), &author, &author, "Initial commit", &tree, &[])
        .expect("Failed to create initial commit");
}
