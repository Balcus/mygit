use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub last_commit_hash: Option<String>,
    pub ref_path: PathBuf,
}
