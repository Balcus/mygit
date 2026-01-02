use flux_core::{commands, repo::repository::Repository, utils};
use serial_test::serial;
use std::fs;

mod common;

#[test]
#[serial]
fn project_creation_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    assert!(project_path.join("README.md").exists());
    assert!(project_path.join("src/main.rs").exists());
    assert!(project_path.join("src/lib.rs").exists());

    let readme = fs::read_to_string("README.md").unwrap();
    let main_rs = fs::read_to_string("src/main.rs").unwrap();
    let lib_rs = fs::read_to_string("src/lib.rs").unwrap();

    assert_eq!(readme, "Read this file before running the project");
    assert_eq!(main_rs, r#"pub fn main() { println!("{}", add(1, 2)) }"#);
    assert_eq!(lib_rs, "pub fn add(a: i32, b: i32) -> i64 { a + b }");
}

#[test]
#[serial]
fn init_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    Repository::init(None, false).unwrap();

    assert!(project_path.join(".flux/config").exists());
    assert!(project_path.join(".flux/HEAD").exists());
    assert!(project_path.join(".flux/objects").exists());
    assert!(project_path.join(".flux/refs").exists());

    let head = fs::read_to_string(".flux/HEAD").unwrap();
    assert_eq!(head, "ref: refs/heads/main\n");
}

#[test]
#[serial]
fn set_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    Repository::init(None, false).unwrap();

    commands::set(None, "user_name".to_string(), "user".to_string()).unwrap();
    commands::set(None, "user_email".to_string(), "user@gmail.com".to_string()).unwrap();

    assert!(project_path.join(".flux/config").exists());

    let config = fs::read_to_string(".flux/config").unwrap();
    assert!(config.contains("user_name = \"user\""));
    assert!(config.contains("user_email = \"user@gmail.com\""));
}

#[test]
#[serial]
fn hash_object_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    Repository::init(None, false).unwrap();

    // file hashing
    let my_hash = commands::hash_object(None, "README.md".to_string(), false).unwrap();
    let git_hash = common::git_hash_object("README.md").unwrap();
    assert_eq!(my_hash, git_hash);
    let object_path = project_path
        .join(".flux/objects")
        .join(&git_hash[..2])
        .join(&git_hash[2..]);
    assert!(!object_path.exists());
    let _ = commands::hash_object(None, "README.md".to_string(), true).unwrap();
    assert!(object_path.exists());

    // dir hashing (git does not support hashing directories directly)
    assert!(project_path.join("src").exists());
    let my_hash = commands::hash_object(None, "src".to_string(), true).unwrap();
    assert_eq!(my_hash, "ac715a76cc52acc719def812525f6ae57b4770a9");
}

#[test]
#[serial]
fn commit_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    // create repo and set user data
    Repository::init(None, false).unwrap();
    commands::set(None, "user_name".to_string(), "Test User".to_string()).unwrap();
    commands::set(None, "user_email".to_string(), "test@example.com".to_string()).unwrap();

    // check if README is correctly added to the index
    let readme_blob_hash = commands::hash_object(None, "README.md".to_string(), false).unwrap();
    let readme_object_path = project_path
        .join(".flux/objects")
        .join(&readme_blob_hash[..2])
        .join(&readme_blob_hash[2..]);
    assert!(!readme_object_path.exists());

    commands::add(None, "README.md".to_string()).unwrap();

    let index = fs::read_to_string(".flux/index").unwrap();
    assert!(index.contains(&format!("\"./README.md\":\"{}\"", readme_blob_hash)));
    assert!(readme_object_path.exists());

    // check if main and lib are correctly added to index
    commands::add(None, "src/main.rs".to_string()).unwrap();
    commands::add(None, "src/lib.rs".to_string()).unwrap();

    let index = fs::read_to_string(".flux/index").unwrap();
    let main_blob_hash = commands::hash_object(None, "src/main.rs".to_string(), false).unwrap();
    let lib_blob_hash = commands::hash_object(None, "src/lib.rs".to_string(), false).unwrap();

    assert!(index.contains(&format!("\"./src/main.rs\":\"{}\"", main_blob_hash)));
    assert!(index.contains(&format!("\"./src/lib.rs\":\"{}\"", lib_blob_hash)));

    let main_object_path = project_path
        .join(".flux/objects")
        .join(&main_blob_hash[..2])
        .join(&main_blob_hash[2..]);
    assert!(main_object_path.exists());

    // check if commit is created correctly
    let commit_hash = commands::commit(None, "Initial commit".to_string()).unwrap();

    assert_eq!(commit_hash.len(), 40);

    let commit_object_path = project_path
        .join(".flux/objects")
        .join(&commit_hash[..2])
        .join(&commit_hash[2..]);
    assert!(commit_object_path.exists());

    let head_content = fs::read_to_string(".flux/HEAD").unwrap();
    assert_eq!(head_content.trim(), "ref: refs/heads/main");

    let main_ref = fs::read_to_string(".flux/refs/heads/main").unwrap();
    assert_eq!(main_ref.trim(), commit_hash);

    let repo = Repository::open(None).unwrap();
    let commit_data = utils::read_object(&repo.store_dir, &commit_hash).unwrap();
    let commit_content = String::from_utf8(commit_data.decompressed_content).unwrap();

    assert!(commit_content.starts_with("tree "));
    assert!(commit_content.contains("author Test User <test@example.com>"));
    assert!(commit_content.contains("committer Test User <test@example.com>"));
    assert!(commit_content.contains("Initial commit"));
    assert!(!commit_content.contains("parent "));

    let tree_line = commit_content.lines().next().unwrap();
    let tree_hash = tree_line.strip_prefix("tree ").unwrap().trim();

    let tree_object_path = project_path
        .join(".flux/objects")
        .join(&tree_hash[..2])
        .join(&tree_hash[2..]);
    assert!(tree_object_path.exists());

    // update README, create second commit and check if parent is set right
    fs::write("README.md", "Updated content for second commit").unwrap();
    commands::add(None, "README.md".to_string()).unwrap();

    let second_commit_hash = commands::commit(None, "Second commit".to_string()).unwrap();

    assert_ne!(commit_hash, second_commit_hash);

    let main_ref = fs::read_to_string(".flux/refs/heads/main").unwrap();
    assert_eq!(main_ref.trim(), second_commit_hash);

    let second_commit_data = utils::read_object(&repo.store_dir, &second_commit_hash).unwrap();
    let second_commit_content = String::from_utf8(second_commit_data.decompressed_content).unwrap();

    assert!(second_commit_content.contains(&format!("parent {}", commit_hash)));
    assert!(second_commit_content.contains("Second commit"));
}
