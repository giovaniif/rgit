use sha1::{Sha1, Digest};
use std::path::Path;
use std::io::Write;
use std::fs;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Read;
use flate2::read::ZlibDecoder;

pub struct Hash(String);

pub enum ObjectType {
    Blob,
    Tree,
}

impl Hash {
    pub fn new(hex: String) -> Self {
        Self(hex)
    }

    pub fn fan_out(&self) -> (&str, &str) {
        self.0.split_at(2)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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
    pub name: String
}

pub struct Commit {
    pub tree_hash: Hash,
    pub parent_hash: Option<Hash>,
    pub author: String,
    pub message: String,
}

fn prepare_blob(content: &[u8]) -> Vec<u8> {
    let header = format!("blob {}\0", content.len());
    let mut data = Vec::with_capacity(header.len() + content.len());
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(content);
    data
}

pub fn hash_data(data: &[u8]) -> Hash {
    let mut hasher = Sha1::new();
    hasher.update(data);
    Hash::new(hex::encode(hasher.finalize()))
}

pub fn hash_blob(content: &[u8]) -> String {
    let header = format!("blob {}\0", content.len());

    let mut data = Vec::new();
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(content);

    let mut hasher = Sha1::new();
    hasher.update(&data);
    let result = hasher.finalize();

    hex::encode(result)
}

pub fn get_object_path(hash: &str) -> (String, String) {
    let (dir, file) = hash.split_at(2);
    (dir.to_string(), file.to_string())
}

pub fn read_blob(repo_root: &Path, hash: &Hash) -> std::io::Result<Vec<u8>> {
    let (prefix, rest) = hash.fan_out();
    let object_path = repo_root.join(".rgit/objects").join(prefix).join(rest);

    let file = fs::File::open(object_path)?;
    let mut decoder = ZlibDecoder::new(file);
    let mut full_contents = Vec::new();
    decoder.read_to_end(&mut full_contents)?;

    if let Some(null_pos) = full_contents.iter().position(|&b| b == 0) {
        Ok(full_contents[null_pos + 1..].to_vec())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid Git object: missing null terminator"
        ))
    }
}

pub fn prepare_tree(entries: &[TreeEntry]) -> Vec<u8> {
    let mut body = Vec::new();

    for entry in entries {
        let line = format!(
            "{} {} {}\t{}\n",
            entry.mode,
            entry.otype.as_str(),
            entry.hash.as_str(),
            entry.name
        );
        body.extend_from_slice(line.as_bytes());
    }

    let header = format!("tree {}\0", body.len());
    let mut full_data = Vec::with_capacity(header.len() + body.len());
    full_data.extend_from_slice(header.as_bytes());
    full_data.extend_from_slice(&body);
    full_data
}

pub fn store_object(repo_root: &Path, full_data: &[u8]) -> std::io::Result<Hash> {
    let hash = hash_data(full_data);
    let (prefix, rest) = hash.fan_out();
    let object_path = repo_root.join(".rgit/objects").join(prefix).join(rest);

    if !object_path.exists() {
        fs::create_dir_all(object_path.parent().unwrap())?;
        let file = fs::File::create(object_path)?;
        let mut encoder = ZlibEncoder::new(file, Compression::default());
        encoder.write_all(full_data)?;
        encoder.finish()?;
    }

    Ok(hash)
}

pub fn store_blob(repo_root: &Path, content: &[u8]) -> std::io::Result<Hash> {
    let data = prepare_blob(content);
    store_object(repo_root, &data)
}

pub fn prepare_commit(commit: &Commit) -> Vec<u8> {
    let mut body = String::new();

    body.push_str(&format!("tree {}\n", commit.tree_hash.as_str()));

    if let Some(parent) = &commit.parent_hash {
        body.push_str(&format!("parent {}\n", parent.as_str()));
    }

    body.push_str(&format!("author {}\n", commit.author));
    body.push_str(&format!("\n{}\n", commit.message));

    let header = format!("commit {}\0", body.len());
    let mut full_data = Vec::with_capacity(header.len() + body.len());
    full_data.extend_from_slice(header.as_bytes());
    full_data.extend_from_slice(body.as_bytes());
    full_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_hashing() {
        let content = "hello world";
        let expected_hash = "95d09f2b10159347eece71399a7e2e907ea3df4f";

        let hash = hash_blob(content.as_bytes());
        assert_eq!(hash, expected_hash);
    }

    #[test]
    fn test_object_storage_path() {
        let hash = "95d09f2b10159347eece71399a7e2e907ea3df4f";
        let (dir, file) = get_object_path(hash);

        assert_eq!(dir, "95");
        assert_eq!(file, "d09f2b10159347eece71399a7e2e907ea3df4f")
    }

    
    use tempfile::tempdir;

    #[test]
    fn test_write_and_read_blob() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();
        let content = b"hello world";

        let hash = store_blob(repo_path, content).expect("Failed to store");

        assert_eq!(hash.as_str(), "95d09f2b10159347eece71399a7e2e907ea3df4f");
    }

    #[test]
    fn test_read_blob_content() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();
        let original_content = b"rust git project";

        let hash = store_blob(repo_path, original_content).unwrap();

        let read_result = read_blob(repo_path, &hash).expect("Failed to read blob");

        assert_eq!(read_result, original_content);
    }
    
    #[test]
    fn test_tree_formatting() {
        let entries = vec![
            TreeEntry {
                mode: "100644".to_string(),
                otype: ObjectType::Blob,
                hash: Hash::new("95d09f2b10159347eece71399a7e2e907ea3df4f".to_string()),
                name: "hello.txt".to_string(),
            },
        ];

        let tree_data = prepare_tree(&entries);

        assert!(tree_data.starts_with(b"tree"));
        assert!(tree_data.windows(9).any(|w| w == b"hello.txt"));
    }

    #[test]
    fn test_tree_with_multiple_blobs() {
       let dir = tempdir().unwrap();
       let repo_path = dir.path();

       let hash_a = store_blob(repo_path, b"content of a").unwrap();
       let hash_b = store_blob(repo_path, b"content of b").unwrap();

        let entries = vec![
            TreeEntry {
                mode: "100644".to_string(),
                otype: ObjectType::Blob,
                hash: hash_a,
                name: "a.txt".to_string(),
            },
            TreeEntry {
                mode: "100644".to_string(),
                otype: ObjectType::Blob,
                hash: hash_b,
                name: "b.txt".to_string(),
            },
        ];

        let tree_data = prepare_tree(&entries);
        let tree_hash = store_object(repo_path, &tree_data).expect("Failed to store tree");

        let (prefix, rest) = tree_hash.fan_out();
        let tree_path = repo_path.join(".rgit/objects").join(prefix).join(rest);

        assert!(tree_path.exists());
        assert_ne!(tree_hash.as_str(), "95d09f2b10159347eece71399a7e2e907ea3df4f");
    }

    #[test]
    fn test_commit_formatting() {
       let tree_hash = Hash::new("95d09f2b10159347eece71399a7e2e907ea3df4f".to_string());
       
       let commit = Commit {
            tree_hash,
            parent_hash: None,
            author: "Giovani <gio@example.com>".to_string(),
            message: "Initial commit".to_string(),
       };

       let commit_data = prepare_commit(&commit);

       assert!(commit_data.starts_with(b"commit"));
       assert!(commit_data.windows(14).any(|w| w == b"Initial commit"));
    }

    #[test]
    fn test_full_git_flow() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();

        let blob_hash = store_blob(repo_path, b"hello rust").unwrap();

        let entries = vec![TreeEntry {
            mode: "100644".to_string(),
            otype: ObjectType::Blob,
            hash: blob_hash,
            name: "main.rs".to_string(),
        }];
        let tree_data = prepare_tree(&entries);
        let tree_hash = store_object(repo_path, &tree_data).unwrap();

        let commit = Commit {
            tree_hash,
            parent_hash: None,
            author: "Developer <dev@rgit.com>".to_string(),
            message:  "First snapshot".to_string(),
        };
        let commit_data = prepare_commit(&commit);
        let commit_hash = store_object(repo_path, &commit_data).unwrap();

        assert!(repo_path.join(".rgit/objects").exists());
        println!("Commit Hash: {}", commit_hash.as_str());
    }
}

