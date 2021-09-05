// from https://siciarz.net/24-days-rust-git2/

use git2::{Oid, Signature, Repository, BranchType, Direction, Commit, ObjectType};
use std::path::Path;
use regex::Regex;

// TODO: return a Result<bool, Error> instead?
pub(crate) fn branch_exists(repo: &Repository, reserve_number: i32) -> bool {
    let pattern = format!("*{}", reserve_number);
    let re = Regex::new(pattern.as_str()).unwrap();
    let c = repo.branches(Some(BranchType::Remote))
        .unwrap()
        .find(|b| re.is_match(b.as_ref().unwrap().0.name().unwrap().unwrap()));

    return c.is_some();
}

pub(crate) fn checkout_branch(repo: &Repository, branch_name: &str) {
    let head = repo.head().unwrap();
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid).unwrap();

    let branch = repo.branch(
        branch_name,
        &commit,
        false,
    );

    let obj = repo.revparse_single(&("refs/heads/".to_owned() + branch_name)).unwrap();

    repo.checkout_tree(
        &obj,
        None
    );
}

pub(crate) fn add_and_commit(repo: &Repository, path: &Path, message: &str) -> Result<Oid, git2::Error> {
    let mut index = repo.index()?;
    index.add_path(path)?;

    // TODO: is this required?
    // index.write()?;

    let oid = index.write_tree()?;
    let parent_commit = find_last_commit(&repo)?;
    let tree = repo.find_tree(oid)?;
    let signature = repo.signature()?;

    return repo.commit(Some("HEAD"), //  point HEAD to our new commit
                &signature, // author
                &signature, // committer
                message, // commit message
                &tree, // tree
                &[&parent_commit]);// parents
}

pub(crate) fn push(repo: &Repository) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote("origin")?;
    remote.connect(Direction::Push)?;
    remote.push(&["refs/heads/master:refs/heads/master"], None)
}

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}
