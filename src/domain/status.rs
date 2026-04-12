use std::collections::HashMap;
use std::path::Path;
use std::fs;
use crate::domain::hash::Hash;
use crate::domain::blob::Blob;

#[derive(Debug, PartialEq)]
pub enum FileState {
    Modified,
    Untracked,
    Deleted,
}

pub struct StatusResult {
    pub changes: HashMap<String, FileState>,
}

pub fn calculate_status(
    repo_root: &Path,
    head_entries: &HashMap<String, Hash>
) -> std::io::Result<StatusResult> {
    let mut changes = HashMap::new();
    let mut seen_on_disk = std::collections::HashSet::new();

    for entry in fs::read_dir(repo_root)? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap();

        if name == ".rgit" || name == "target" || name.starts_with('.') {
            continue;
        }

        seen_on_disk.insert(name.clone());

        match head_entries.get(&name) {
            Some(last_hash) => {
                let content = fs::read(entry.path())?;
                let current_hash = Hash::from_bytes(&Blob::prepare(&content));
                if current_hash.as_str() != last_hash.as_str() {
                    changes.insert(name, FileState::Modified);
                }
            }
            None => {
                changes.insert(name, FileState::Untracked);
            }
        }
    }

    for head_name in head_entries.keys() {
        if !seen_on_disk.contains(head_name) {
            changes.insert(head_name.clone(), FileState::Deleted);
        }
    }

    Ok(StatusResult { changes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::blob::Blob;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_calculate_status() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();

        let content_v1 = b"version 1";
        let blob_hash = Blob::store(repo_path, content_v1).unwrap();

        let mut head_entries = HashMap::new();
        head_entries.insert("file_a.txt".to_string(), blob_hash);

        fs::write(repo_path.join("file_a.txt"), "version 2").unwrap();
        fs::write(repo_path.join("file_b.txt"), "new file").unwrap();

        let result = calculate_status(repo_path, &head_entries).unwrap();

        assert_eq!(result.changes.get("file_a.txt"), Some(&FileState::Modified));
        assert_eq!(result.changes.get("file_b.txt"), Some(&FileState::Untracked));
    }
}

