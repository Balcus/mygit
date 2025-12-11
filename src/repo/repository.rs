use std::fs;
use std::path::{Path, PathBuf};
use anyhow::bail;
use crate::repo::config::Config;

pub struct Repository {
    pub work_tree: PathBuf,
    pub git_dir: PathBuf,
    pub config: Config
}

impl Repository {
    pub fn init<P: AsRef<Path>>(path: Option<P>) -> Result<(), anyhow::Error>{
        let work_tree = match &path {
            Some(p) => p.as_ref().to_path_buf(),
            None => PathBuf::from("."),
        };

        let git_dir = work_tree.join(".git");

        if git_dir.exists() {
            println!("Repository already initialized");
            return Ok(());
        }

        fs::create_dir(&git_dir)?;

        let config_path = &git_dir.join("config");
        let _ = Config::default(config_path);

        fs::create_dir(&git_dir.join("objects"))?;
        fs::create_dir(&git_dir.join("refs"))?;
        fs::write(&git_dir.join("HEAD"), "ref: refs/heads/main\n")?;

        println!("Initialized repository");

        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: Option<P>) -> Result<Self, anyhow::Error> {
        let work_tree = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => PathBuf::from(".")
        };

        let git_dir = work_tree.join(".git");

        if !git_dir.exists() {
            bail!("Not a git repository");
        }

        let config_path = git_dir.join("config");

        Ok(Self{
            config: Config::from(&config_path)?,
            work_tree,
            git_dir,
        })
    }

}