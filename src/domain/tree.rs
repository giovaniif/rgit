use std::{collections::HashMap, fs, path::Path};

use crate::{domain::{hash::Hash}, store::object_store};

pub enum ObjectType { Blob, Tree }

impl ObjectType {
    pub fn as_str(&self) -> &str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
        }
    }
}

pub struct TreeEntry {
    pub mode: String,
    pub otype: ObjectType,
    pub hash: Hash,
    pub name: String,
}

pub struct Tree;

impl Tree {
    pub fn prepare(entries: &[TreeEntry]) -> Vec<u8> {
        let mut body = Vec::new();
        for entry in entries {
            let line = format!("{} {} {}\t{}\n", entry.mode, entry.otype.as_str(), entry.hash.as_str(), entry.name);
            body.extend_from_slice(line.as_bytes());
        }
        let header = format!("tree {}\0", body.len());
        let mut full_data = Vec::with_capacity(header.len() + body.len());
        full_data.extend_from_slice(header.as_bytes());
        full_data.extend_from_slice(&body);
        full_data
    }

        
    pub fn parse(data: &[u8]) -> Vec<TreeEntry> {
        let s = String::from_utf8_lossy(data);
        let mut entries = Vec::new();

        for line in s.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            let metadata: Vec<&str> = parts[0].split_whitespace().collect();

            if parts.len() == 2 && metadata.len() == 3 {
                entries.push(TreeEntry {
                    mode:  metadata[0].to_string(),
                    otype: if metadata[1] == "blob" { ObjectType::Blob } else { ObjectType::Tree },
                    hash: Hash::new(metadata[2].to_string()),
                    name: parts[1].to_string()
                })
            }
        }

        entries
    }

    pub fn write_from_path(repo_root: &Path) -> std::io::Result<Hash> {
        let mut entries = Vec::new();

        for entry in fs::read_dir(repo_root)? {
            let entry = entry?;
            let name = entry.file_name().into_string().unwrap();
            let path = entry.path();

            if name == ".rgit" || name == "target" { continue; }

            if path.is_file() {
                let content = fs::read(&path)?;
                let blob_hash = crate::domain::blob::Blob::store(repo_root, &content)?;

                entries.push(TreeEntry {
                    mode: "100644".to_string(),
                    otype: ObjectType::Blob,
                    hash: blob_hash,
                    name,
                });
            }
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        let data = Self::prepare(&entries);
        object_store::write(repo_root, &data)
    }

    pub fn get_entries_map(repo_root: &Path, tree_hash: &Hash) -> std::io::Result<HashMap<String, Hash>> {
        let data = object_store::read(repo_root, tree_hash)?;
        let null_pos = data.iter().position(|&b| b == 0).unwrap();
        let entries = Self::parse(&data[null_pos + 1..]);

        let mut map = HashMap::new();
        for entry in entries {
            map.insert(entry.name, entry.hash);
        }
        Ok(map)
    }
}
