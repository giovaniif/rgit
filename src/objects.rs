use sha1::{Sha1, Digest};
use std::path::Path;
use std::io::Write;
use std::fs;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Read;
use flate2::read::ZlibDecoder;

pub struct Hash(String);

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

pub fn store_blob(repo_root: &Path, content: &[u8]) -> std::io::Result<Hash> {
    let full_data = prepare_blob(content);
    let hash = hash_data(&full_data);

    let (prefix, rest) = hash.fan_out();
    let object_path = repo_root.join(".rgit/objects").join(prefix).join(rest);

    if !object_path.exists() {
        fs::create_dir_all(object_path.parent().unwrap())?;
        let file = fs::File::create(object_path)?;
        let mut encoder = ZlibEncoder::new(file, Compression::default());
        encoder.write_all(&full_data)?;
        encoder.finish()?;
    }

    Ok(hash)
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
}

