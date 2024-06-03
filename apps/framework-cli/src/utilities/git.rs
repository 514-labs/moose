use std::path::Path;
use std::sync::Arc;

use crate::framework::languages::SupportedLanguages;
use git2::{
    Error, ErrorClass, ErrorCode, ObjectType, Repository, RepositoryInitOptions, Signature,
};
use log::warn;

use crate::project::Project;

use super::constants::{CLI_USER_DIRECTORY, GITIGNORE};

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

pub fn dump_old_version_schema(
    project: &Project,
    commit_hash: String,
    dest: &Path,
) -> Result<(), Error> {
    let repo = Repository::discover(project.project_location.clone())?;

    let schema_dir = project.schemas_dir();
    let schema_relative_path_from_repo_root = schema_dir
        .strip_prefix(repo.path().parent().unwrap())
        .unwrap();

    let commit = repo.revparse_single(&commit_hash)?;
    let commit = commit.as_commit().ok_or(Error::from_str(&format!(
        "Object {} is not a commit",
        commit_hash
    )))?;

    let tree = commit
        .tree()?
        .get_path(schema_relative_path_from_repo_root)?
        .to_object(&repo)?;
    let tree = tree.as_tree().ok_or(Error::from_str(&format!(
        "Object {} is not a tree",
        tree.id()
    )))?;

    recursive_dump_tree_content(&repo, tree, dest)?;
    Ok(())
}

fn recursive_dump_tree_content(
    repo: &Repository,
    tree: &git2::Tree,
    dest: &Path,
) -> Result<(), Error> {
    for entry in tree.iter() {
        let entry_name = entry.name().ok_or(Error::from_str(&format!(
            "Invalid UTF-8 filename {:?}",
            entry.name_bytes()
        )))?;
        let entry_dest = dest.join(entry_name);

        match entry.kind() {
            Some(ObjectType::Tree) => {
                let tree = repo.find_tree(entry.id())?;
                std::fs::create_dir_all(&entry_dest).map_err(file_system_error_to_git_error)?;
                recursive_dump_tree_content(repo, &tree, &entry_dest)?;
            }
            Some(ObjectType::Blob) => {
                let blob = repo.find_blob(entry.id())?;
                std::fs::write(entry_dest, blob.content())
                    .map_err(file_system_error_to_git_error)?;
            }
            kind => warn!("Unknown kind: {:?} for entry name {:?}", kind, entry_name),
        }
    }
    Ok(())
}

pub fn current_commit_hash(project: &Project) -> Result<String, Error> {
    let repo = Repository::discover(project.project_location.clone())?;
    let head = repo.head()?;
    let mut hash = head.target().unwrap().to_string();
    hash.truncate(7);
    Ok(hash)
}

fn file_system_error_to_git_error(err: std::io::Error) -> Error {
    Error::new(
        ErrorCode::GenericError,
        ErrorClass::Filesystem,
        err.to_string(),
    )
}
