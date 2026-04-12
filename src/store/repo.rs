use std::path::{Path, PathBuf};
use std::fs;

use crate::domain::hash::Hash;

pub struct Repo {
    pub root: PathBuf,
}

impl Repo {
    pub fn new(path: &Path) -> Self {
        Self { root: path.to_path_buf() }
    }

    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.root.join(".rgit/objects"))?;
        fs::create_dir_all(self.root.join(".rgit/refs/heads"))?;
        fs::write(self.root.join(".rgit/HEAD"), "ref: refs/heads/main\n")?;
        Ok(())
    }

    pub fn get_head_hash(&self) -> Option<String> {
        let main_ref = self.root.join(".rgit/refs/heads/main");
        if main_ref.exists() {
            Some(fs::read_to_string(main_ref).ok()?.trim().to_string())
        } else {
            None
        }
    }

    pub fn update_head(&self, hash: &Hash) -> std::io::Result<()> {
        let main_ref = self.root.join(".rgit/refs/heads/main");
        if let Some(parent) = main_ref.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(main_ref, format!("{}\n", hash.as_str()))
    }
}
