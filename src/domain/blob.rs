use crate::domain::hash::Hash;
use crate::store::object_store;
use std::path::Path;

pub struct Blob;

impl Blob {
    pub fn prepare(content: &[u8]) -> Vec<u8> {
        let header = format!("blob {}\0", content.len());
        let mut data = Vec::with_capacity(header.len() + content.len());
        data.extend_from_slice(header.as_bytes());
        data.extend_from_slice(content);
        data
    }

    pub fn store(repo_root: &Path, content: &[u8]) -> std::io::Result<Hash> {
        let full_data = Self::prepare(content);
        object_store::write(repo_root, &full_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_preparation() {
        let data = b"hello";
        let prepared = Blob::prepare(data);
        assert_eq!(prepared, b"blob 5\0hello");
    }
}
