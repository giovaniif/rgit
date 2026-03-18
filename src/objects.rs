use sha1::{Sha1, Digest};
use std::path::Path;
use std::io::Write;
use std::fs;
use std::fs::File;
use flate2::write::ZlibEncoder;
use flate2::Compression;

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

pub fn store_blob(repo_root: &Path, content: &[u8]) -> std::io::Result<String> {
    let header = format!("blob {}\0", content.len());
    let mut full_data = Vec::new();
    full_data.extend_from_slice(header.as_bytes());
    full_data.extend_from_slice(content);

    let hash = hash_blob(content);
    let (prefix, rest) = hash.split_at(2);

    let object_dir = repo_root.join(".rgit/objects").join(prefix);
    fs::create_dir_all(&object_dir)?;

    let object_path = object_dir.join(rest);

    let file = File::create(object_path)?;
    let mut encoder = ZlibEncoder::new(file, Compression::default());
    encoder.write_all(&full_data)?;
    encoder.finish()?;

    Ok(hash)
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

        let hash = store_blob(repo_path, content).expect("Failed to store blob");

        let (prefix, rest) = hash.split_at(2);
        let object_path = repo_path.join(".rgit/objects").join(prefix).join(rest);

        assert!(object_path.exists());
    }
}

