use crate::objects::{commit, tree};
use crate::repo::config::Config;
use crate::repo::index::Index;
use crate::shared::types::object_type::ObjectType;
use crate::shared::types::tree_entry::TreeEntry;
use crate::utils;
use crate::utils::write_object;
use anyhow::{Context, bail};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Repository {
    pub work_tree: PathBuf,
    pub git_dir: PathBuf,
    pub config: Config,
    pub index: Index,
}

impl Repository {
    pub fn init(path: Option<String>, force: bool) -> anyhow::Result<Self> {
        let work_tree = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let git_dir = work_tree.join(".git");

        if git_dir.join("config").exists() && !force {
            bail!("Repository already initialized");
        }

        fs::create_dir_all(&git_dir)?;
        fs::create_dir(&git_dir.join("objects"))?;
        fs::create_dir(&git_dir.join("refs"))?;
        let config = Config::default(&git_dir.join("config"))?;
        fs::write(&git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
        fs::write(&git_dir.join("index"), "{}")?;
        let index = Index::empty(&git_dir)?;
        println!("Initialized repository");

        Ok(Self {
            work_tree,
            index,
            git_dir,
            config,
        })
    }

    pub fn open(path: Option<String>) -> anyhow::Result<Self> {
        let work_tree = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let git_dir = work_tree.join(".git");

        if !git_dir.exists() {
            bail!("Not a git repository");
        }

        let config_path = git_dir.join("config");
        let config = Config::from(&config_path)?;
        let index = Index::load(&git_dir)?;

        Ok(Self {
            config,
            work_tree,
            git_dir,
            index
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), anyhow::Error> {
        self.config.set(key, value)?;
        Ok(())
    }

    pub fn hash_object(&self, path: String, write: bool) -> anyhow::Result<String> {
        let full_path = self.work_tree.join(&path);
        let hash = if write {
            let result = write_object(&self.git_dir, &self.work_tree, &full_path)?;
            result.hash
        } else {
            utils::get_hash(&self.git_dir, &self.work_tree, &full_path)?
        };
        
        Ok(hash)
    }

    pub fn cat_file(&self, object_hash: String) -> anyhow::Result<()> {
        let object = utils::read_object(&self.git_dir, &object_hash)?;

        match object.object_type {
            ObjectType::Blob => {
                let output = String::from_utf8(object.decompressed_content)
                    .context("Blob contains invalid UTF-8")?;
                print!("{}", output);
            }
            ObjectType::Tree => {
                self.ls_tree(object_hash)?;
            }
            ObjectType::Commit => {
                commit::show_commit(&self.git_dir, object_hash)?;
            }
            _ => bail!("cat_file currently supports only blob objects"),
        }

        Ok(())
    }

    pub fn ls_tree(&self, tree_hash: String) -> anyhow::Result<()> {
        let object = utils::read_object(&self.git_dir, &tree_hash)?;

        match object.object_type {
            ObjectType::Tree => {
                let mut result: String = String::new();
                let mut i = 0;
                let content = object.decompressed_content;

                while i < content.len() {
                    let mode_end = content[i..].iter().position(|&b| b == b' ').unwrap();
                    let mode = std::str::from_utf8(&content[i..i + mode_end])?;
                    i += mode_end + 1;

                    let name_end = content[i..].iter().position(|&b| b == b'\0').unwrap();
                    let name = std::str::from_utf8(&content[i..i + name_end])?;
                    i += name_end + 1;

                    let hash = hex::encode(&content[i..i + 20]);
                    i += 20;

                    let object_type = if mode.starts_with("040") {
                        "tree"
                    } else {
                        "blob"
                    };

                    result.push_str(&format!("{mode} {object_type} {hash} {name}\n"));
                }

                print!("{}", result);
            }
            _ => bail!("ls_tree requires a tree object"),
        }
        Ok(())
    }

    pub fn commit_tree(&self, tree_hash: String, message: String, parent_hash: Option<String>) -> anyhow::Result<String> {
        let user_name =
            self.config.user_name.clone().context(
                "Please configure user settings (user_name) in order to create a commit",
            )?;

        let user_email =
            self.config.user_email.clone().context(
                "Please configure user settings (user_email) in order to create a commit",
            )?;

        let object = utils::read_object(&self.git_dir, &tree_hash)?;

        let hash = match object.object_type {
            ObjectType::Tree => commit::commit_tree(
                &self.git_dir,
                user_name,
                user_email,
                tree_hash,
                parent_hash,
                message,
            )?,
            _ => bail!("Can only commit tree objects"),
        };
        Ok(hash)
    }

    pub fn add(&mut self, path: &str) -> anyhow::Result<()> {
        let path = self.work_tree.join(path);
        let res = utils::write_object(&self.git_dir, &self.work_tree, &path)?;
        if let Some(s) = path.to_str() {
            self.index.add(s.into(), res.hash)?;
            self.index.flush()?;
        }else {
            bail!("Could not add file to index.")
        }
        
        Ok(())
    }

    pub fn delete(&mut self, path: &str) -> anyhow::Result<()> {
        let path = self.work_tree.join(path);
        if let Some(s) = path.to_str() {
            self.index.remove(s.into())?;
            self.index.flush()?;
        }else {
            bail!("Could not remove file from index.")
        }
        
        Ok(())
    }

    pub fn tree_from_index(&self) -> anyhow::Result<String> {
        let mut entries = Vec::new();
        
        for (path, hash) in &self.index.map {
            let name = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .context("Invalid filename in index")?
                .to_string();
            
            let object = utils::read_object(&self.git_dir, hash)?;
            
            let (mode, entry_type) = match object.object_type {
                ObjectType::Blob => ("100644".to_string(), "blob".to_string()),
                ObjectType::Tree => ("040000".to_string(), "tree".to_string()),
                _ => bail!("Unexpected object type in index"),
            };
            
            entries.push(TreeEntry {
                mode,
                entry_type,
                hash: hash.clone(),
                name,
            });
        }
        
        let tree_content = tree::build_tree_content(entries);
        let result = tree::hash_tree(tree_content)?;
        utils::store_object(&self.git_dir, &result.object_hash, &result.compressed_content)?;
        
        Ok(result.object_hash)
    }
}
