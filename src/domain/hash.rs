use hex;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        Self(hex::encode(hasher.finalize()))
    }

    pub fn get_path(&self, repo_root: &Path) -> PathBuf {
        let (prefix, rest) = self.fan_out();
        repo_root.join(".rgit/objects").join(prefix).join(rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_fan_out() {
        let hash = Hash::new("4a2b1c3d".to_string());
        let (prefix, rest) = hash.fan_out();
        assert_eq!(prefix, "4a");
        assert_eq!(rest, "2b1c3d");
    }
}
