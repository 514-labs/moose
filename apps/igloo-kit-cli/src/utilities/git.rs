use std::path::Path;

use crate::framework::languages::SupportedLanguages;
use git2::{Error, Repository, Signature};

use crate::project::Project;

pub fn is_git_repo(dir_path: &Path) -> Result<bool, Error> {
    match Repository::open(dir_path) {
        Ok(_) => Ok(true),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn create_init_commit(project: &Project, dir_path: &Path) {
    let mut git_ignore_file = project.project_file_location.clone();
    git_ignore_file.pop();
    git_ignore_file.push(".gitignore");
    let mut git_ignore_entries = vec![".igloo"];
    git_ignore_entries.append(&mut match project.language {
        SupportedLanguages::Typescript => vec!["node_modules", "dist", "coverage"],
    });
    let mut git_ignore = git_ignore_entries.join("\n");
    git_ignore.push_str("\n\n");
    std::fs::write(git_ignore_file, git_ignore).unwrap();

    let repo = Repository::init(dir_path).expect("Failed to initialize git repo");
    let author =
        Signature::now("Moose CLI", "noreply@fiveonefour.com").expect("Failed to create signature");

    // Now let's create an empty tree for this commit
    let mut index = repo.index().expect("Failed to get repo index");
    index
        .add_all(&["."], git2::IndexAddOption::DEFAULT, None)
        .expect("Failed to add path to index");
    index.write().expect("Failed to write index");
    let tree_id = index.write_tree().expect("Failed to write tree");

    let tree = repo.find_tree(tree_id).expect("Failed to find tree");

    // empty parent because it's the first commit
    repo.commit(Some("HEAD"), &author, &author, "Initial commit", &tree, &[])
        .expect("Failed to create initial commit");
}
